mod build;
mod clangd;
mod help;
mod new;
mod test;

use crate::{
    config::{BuildFile, ToolChain},
    error::Error,
    input::BuildSwitches,
    log_info_ln,
};
pub use build::build;
pub use clangd::clangd;
pub use help::{help, version};
pub use new::{init, new};
use std::{path::PathBuf, process::ExitCode};
pub use test::test;

pub fn clean(build: &BuildFile) -> Result<(), Error> {
    log_info_ln!("cleaning build files for \"{}\"", build.name);
    match std::fs::remove_dir_all("bin") {
        Ok(()) => (),
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(Error::FileSystem(e));
            }
        }
    }
    Ok(())
}

pub fn run(name: &str, switches: &BuildSwitches, runargs: Vec<String>) -> Result<ExitCode, Error> {
    let outdir = if switches.toolchain == ToolChain::system_default() {
        PathBuf::from("bin").join(switches.profile.to_string())
    } else {
        PathBuf::from("bin")
            .join(switches.toolchain.as_directory())
            .join(switches.profile.to_string())
    };
    let outfile = outdir.join(name).with_extension(switches.toolchain.app_ext());

    log_info_ln!("{:=<80}", format!("running application: {} ", outfile.display()));
    let code = std::process::Command::new(PathBuf::from(".").join(&outfile))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .map_err(|_| Error::InvalidExe(outfile.clone()))?
        .code()
        .ok_or(Error::ExeKilled(outfile.clone()))?;

    const WIN_KILLPROC: i32 = -0x3FFFFEC6;
    const WIN_SEGFAULT: i32 = -0x3FFFFFFB;
    const WIN_OVERFLOW: i32 = -0x3FFFFBF7;

    if cfg!(windows) && (code == WIN_KILLPROC || code == WIN_SEGFAULT || code == WIN_OVERFLOW) {
        return Err(Error::ExeKilled(outfile));
    }

    let code: u8 = code
        .try_into()
        .unwrap_or(1);

    Ok(code.into())
}
