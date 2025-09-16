use std::io::Write;
use crate::{
    config::{VangoFile, BuildFile, LibFile, ProjKind, Profile, Dependency, WarnLevel},
    error::Error,
    log_info_ln,
};


pub fn clangd(build: &BuildFile, block_output: bool) -> Result<(), Error> {
    if !block_output {
        log_info_ln!("generating 'compile_flags.txt' for '{}'", build.name);
    }
    let mut file = std::io::BufWriter::new(std::fs::File::create("compile_flags.txt")?);
    writeln!(file, "-std={}", build.lang)?;
    if build.lang.is_cpp() {
        writeln!(file, "-xc++")?;
    }

    let profile = build.get(&Profile::Debug)?;
    match profile.settings.warn_level {
        WarnLevel::None  => writeln!(file, "-w")?,
        WarnLevel::Basic => writeln!(file, "-Wall")?,
        WarnLevel::High  => writeln!(file, "\
-Wall
-Wextra
-Wconversion
-Wsign-conversion
-Wshadow
-Wformat=2
-Wnull-dereference
-Wdouble-promotion
-Wimplicit-fallthrough")?,
    }
    if profile.settings.iso_compliant {
        writeln!(file, "-Wpedantic")?;
    }

    let home = std::env::home_dir().unwrap();
    let mut defines = Vec::new();
    let mut incdirs = Vec::new();

    for lib in &build.dependencies {
        let path = match lib {
            Dependency::Local { path, .. } => path.clone(),
            Dependency::Git { git, tag, recipe, .. } => {
                let git = std::path::Path::new(&git);
                let stem = git.file_stem().unwrap().to_string_lossy();
                let path = home.join(format!(".vango/packages/{stem}"));
                if !std::fs::exists(&path).unwrap() {
                    crate::fetch::pull_git_repo(git, tag, recipe, &path);
                }
                path.clone()
            }
            Dependency::Headers { headers, .. } => {
                incdirs.push(headers.clone());
                continue;
            }
            Dependency::System{..} => continue,
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path))
        }

        let save = std::env::current_dir().unwrap();
        std::env::set_current_dir(&path).unwrap();
        let mut library = match VangoFile::from_str(&crate::read_manifest()?)? {
            VangoFile::Build(build) => LibFile::try_from(build)?,
            VangoFile::Lib(lib) => lib,
        };
        std::env::set_current_dir(&save).unwrap();
        let profile = library.take(&Profile::Debug)?;
        defines.extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
        incdirs.push(path.join(profile.include));
    }

    for def in defines {
        writeln!(file, "-D{def}")?;
    }
    if cfg!(windows) {
        writeln!(file, "-DUNICODE")?;
        writeln!(file, "-D_UNICODE")?;
        if let ProjKind::SharedLib{..} = build.kind {
            writeln!(file, "-DVANGO_EXPORT_SHARED")?;
        }
    }
    writeln!(file, "-DVANGO_DEBUG")?;
    writeln!(file, "-DVANGO_PKG_NAME=\"{}\"", build.name)?;
    writeln!(file, "-DVANGO_PKG_VERSION=\"{}\"", build.version)?;
    writeln!(file, "-DVANGO_PKG_VERSION_MAJOR={}", build.version.major)?;
    writeln!(file, "-DVANGO_PKG_VERSION_MINOR={}", build.version.minor)?;
    writeln!(file, "-DVANGO_PKG_VERSION_PATCH={}", build.version.patch)?;
    for inc in incdirs {
        writeln!(file, "-I{}", inc.display())?;
    }
    for inc in &profile.include {
        writeln!(file, "-I{}", inc.display())?;
    }
    writeln!(file, "-I{}", std::env::current_exe()?
        .parent()
        .unwrap()
        .to_owned()
        .join("testframework")
        .display())?;

    file.flush()?;

    Ok(())
}

