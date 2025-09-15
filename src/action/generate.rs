use std::io::Write;
use crate::{
    config::{VangoFile, BuildFile, LibFile, ProjKind, Profile, Dependency, WarnLevel},
    error::Error,
    log_info_ln,
};


pub fn generate(build: &BuildFile) -> Result<(), Error> {
    log_info_ln!("generating 'compile_flags.txt' for '{}'", build.name);
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

    let mut defines = Vec::new();
    let mut incdirs = Vec::new();

    for lib in &build.dependencies {
        let path = match lib {
            Dependency::Local { path, .. } => path.clone(),
            #[allow(unused)]
            Dependency::Git { git, tag, .. } => continue,
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
        match VangoFile::from_str(&crate::read_manifest()?)? {
            VangoFile::Build(build) => {
                let profile = LibFile::try_from(build)?
                    .take(&Profile::Debug)?;
                defines.extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
                incdirs.push(path.join(profile.include));
            }
            VangoFile::Lib(mut lib) => {
                let profile = lib.take(&Profile::Debug)?;
                defines.extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
                incdirs.push(path.join(profile.include));
            }
        }
        std::env::set_current_dir(&save).unwrap();
    }

    for dep in defines {
        writeln!(file, "-D{dep}")?;
    }
    if cfg!(windows) {
        writeln!(file, "-DUNICODE")?;
        writeln!(file, "-D_UNICODE")?;
        if let ProjKind::SharedLib{..} = build.kind {
            writeln!(file, "-DVANGO_EXPORT_SHARED")?;
        }
    }
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

