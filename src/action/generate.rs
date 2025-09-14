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
        WarnLevel::None => {
            writeln!(file, "-w")?;
            if profile.settings.iso_compliant {
                writeln!(file, "-Wpedantic")?;
            }
        }
        WarnLevel::Basic => {
            writeln!(file, "-Wall")?;
            if profile.settings.iso_compliant {
                writeln!(file, "-Wpedantic")?;
            }
        }
        WarnLevel::High => {
            writeln!(file, "\
                -Wall
                -Wextra
                -Wpedantic
                -Wconversion
                -Wsign-conversion
                -Wshadow
                -Wformat=2
                -Wnull-dereference
                -Wdouble-promotion
                -Wimplicit-fallthrough")?;
        }
    }

    let mut defines = Vec::new();
    let mut incdirs = Vec::new();

    for lib in &build.dependencies {
        let path = match lib {
            Dependency::Local { path, .. } => {
                path.clone()
            }
            #[allow(unused)]
            Dependency::Git { git, tag, .. } => {
                continue;
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

        if let Some(build) = if cfg!(windows) && std::fs::exists(path.join("win.vango.toml"))? {
            std::fs::read_to_string(path.join("win.vango.toml")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(path.join("lnx.vango.toml"))? {
            std::fs::read_to_string(path.join("lnx.vango.toml")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(path.join("mac.vango.toml"))? {
            std::fs::read_to_string(path.join("mac.vango.toml")).ok()
        } else {
            std::fs::read_to_string(path.join("vango.toml")).ok()
        } {
            match VangoFile::from_str(&build)? {
                VangoFile::Build(build) => {
                    let mut libinfo = LibFile::try_from(build)?;
                    let profile = libinfo.take(&Profile::Debug)?;
                    defines.extend(profile.defines);
                    incdirs.push(path.join(profile.include));
                }
                VangoFile::Lib(mut lib) => {
                    let profile = lib.take(&Profile::Debug)?;
                    defines.extend(profile.defines);
                    incdirs.push(path.join(profile.include));
                }
            }
        } else {
            return Err(Error::MissingBuildScript(path))
        }
    }

    if cfg!(windows) {
        defines.push("UNICODE".to_string());
        defines.push("_UNICODE".to_string());
        if let ProjKind::SharedLib{..} = build.kind {
            defines.push("VANGO_EXPORT_SHARED".to_string());
        }
    }
    for dep in defines {
        writeln!(file, "-D{dep}")?;
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

