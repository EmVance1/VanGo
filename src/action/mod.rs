mod new;
mod build;
mod run;
mod test;
mod clangd;
mod help;

use crate::{
    config::BuildFile,
    error::Error,
    log_info_ln,
};
pub use new::{init, new};
pub use build::build;
pub use run::run;
pub use test::test;
pub use clangd::clangd;
pub use help::{help, version};

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
