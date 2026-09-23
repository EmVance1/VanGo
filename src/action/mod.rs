mod build;
mod clangd;
mod help;
mod new;
mod run;
mod test;

use crate::{config::PackageManifest, error::Error, log_info_ln};
pub use build::build;
pub use clangd::clangd;
pub use help::{help, version};
pub use new::init;
pub use run::run;
pub use test::test;

pub fn clean(build: &PackageManifest) -> Result<(), Error> {
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
