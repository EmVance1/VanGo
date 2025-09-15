mod new;
mod help;
mod build;
mod test;
mod clangd;

use std::{path::PathBuf, process::ExitCode};
use crate::{
    config::BuildFile, input::BuildSwitches, error::Error, log_info_ln
};
pub use new::{init, new};
pub use help::help;
pub use build::build;
pub use test::test;
pub use clangd::clangd;


pub fn clean(build: &BuildFile) -> Result<(), Error> {
    log_info_ln!("cleaning build files for \"{}\"", build.name);
    match std::fs::remove_dir_all("bin") {
        Ok(()) => (),
        Err(e) => if e.kind() != std::io::ErrorKind::NotFound {
            return Err(Error::FileSystem(e));
        }
    }
    Ok(())
}

pub fn run(name: &str, switches: &BuildSwitches, runargs: Vec<String>) -> Result<ExitCode, Error> {
    let outfile = PathBuf::from("bin")
        .join(switches.profile.to_string()).join(name)
        .with_extension(switches.toolchain.app_ext());

    log_info_ln!("{:=<80}", format!("running application: {} ", outfile.display()));
    let code: u8 = std::process::Command::new(PathBuf::from(".").join(&outfile))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .map_err(|_| Error::InvalidExe(outfile.clone()))?
        .code()
        .ok_or(Error::ExeKilled(outfile))?
        .try_into()
        .unwrap_or(1);

    Ok(code.into())
}

