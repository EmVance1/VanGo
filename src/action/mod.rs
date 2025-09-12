mod new;
mod help;
mod build;
mod test;
mod generate;

use std::{io::Write, path::{Path, PathBuf}};
use crate::{
    config::BuildFile,
    error::Error,
    log_info_ln,
};
pub use new::{init, new};
pub use help::help;
pub use build::build;
pub use test::test;
pub use generate::generate;



pub fn clean(build: &BuildFile) -> Result<(), Error> {
    log_info_ln!("cleaning build files for \"{}\"", build.name);
    match std::fs::remove_dir_all("bin/debug") {
        Ok(()) => (),
        Err(e) => if e.kind() != std::io::ErrorKind::NotFound {
            return Err(Error::FileSystem(e));
        }
    }
    match std::fs::remove_dir_all("bin/release") {
        Ok(()) => (),
        Err(e) => if e.kind() != std::io::ErrorKind::NotFound {
            return Err(Error::FileSystem(e));
        }
    }
    Ok(())
}

pub fn run(outfile: &Path, runargs: Vec<String>) -> Result<u8, Error> {
    log_info_ln!("running application {:=<63}", format!("\"{}\" ", outfile.display()));
    Ok(std::process::Command::new(PathBuf::from(".").join(outfile))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .map_err(|_| Error::InvalidExe(outfile.to_owned()))?
        .code()
        .ok_or(Error::ExeKilled(outfile.to_owned()))? as u8)
}

