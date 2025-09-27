mod error;
mod exec;
mod fetch;
mod input;
// mod input2;
mod config;
mod action;
#[macro_use]
mod log;

use error::Error;
use std::process::ExitCode;


macro_rules! exit_failure {
    ($($arg:tt)*) => { {
        log_error_ln!($($arg)*);
        std::process::exit(1);
    } };
}


fn read_manifest() -> Result<String, Error> {
    let prefix = if cfg!(windows) {
        "win."
    } else if cfg!(target_os = "linux") {
        "lnx."
    } else if cfg!(target_os = "macos") {
        "mac."
    } else { "" };

    let os1 = format!("{prefix}Vango.toml");
    let os2 = format!("{prefix}vango.toml");

    let def1 = "Vango.toml";
    let def2 = "vango.toml";

    if std::fs::exists(&os1).unwrap() {
        Ok(std::fs::read_to_string(&os1)?)
    } else if std::fs::exists(&os2).unwrap() {
        Ok(std::fs::read_to_string(&os2)?)
    } else if std::fs::exists(def1).unwrap() {
        Ok(std::fs::read_to_string(def1)?)
    } else if std::fs::exists(def2).unwrap() {
        Ok(std::fs::read_to_string(def2)?)
    } else {
        Err(Error::MissingBuildScript(std::env::current_dir().unwrap().file_name().unwrap().into()))
    }
}


fn main() -> ExitCode {
    let cmd = input::collect_args().unwrap_or_else(|e| exit_failure!("{}", e));

    if let input::Action::Version = &cmd {
        action::version();
    } else if let input::Action::Help{ action } = &cmd {
        action::help(action.as_ref());
    } else if let input::Action::New { library, strict, is_c, clangd, name } = &cmd {
        action::new(*library, *strict, *is_c, *clangd, name).unwrap_or_else(|e| exit_failure!("{}", e));
    } else if let input::Action::Init{ library, strict, is_c, clangd } = &cmd {
        action::init(*library, *strict, *is_c, *clangd).unwrap_or_else(|e| exit_failure!("{}", e));

    } else {
        let bfile = read_manifest()
            .unwrap_or_else(|e| exit_failure!("{}", e));
        let build = config::VangoFile::from_str(&bfile)
            .unwrap_or_else(|e| exit_failure!("{}", e))
            .get_build()
            .unwrap_or_else(|| exit_failure!("action requires source code ([package]) type manifest"));

        match cmd {
            input::Action::Build{ switches } => {
                action::build(&build, &switches, false).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            input::Action::Run{ switches, args } => {
                if build.kind.is_lib() { exit_failure!("{}", Error::LibNotExe(build.name)); }
                action::build(&build, &switches, false).unwrap_or_else(|e| exit_failure!("{}", e));
                return action::run(&build.name, &switches, args).unwrap_or_else(|e| exit_failure!("{}", e))
            }
            input::Action::Test{ switches, args } => {
                action::build(&build, &switches, true).unwrap_or_else(|e| exit_failure!("{}", e));
                return action::test(build, &switches, args).unwrap_or_else(|e| exit_failure!("{}", e))
            }
            input::Action::Clean => {
                action::clean(&build).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            input::Action::Clangd => {
                action::clangd(&build, false).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            _ => unreachable!(),
        }
    }

    0.into()
}

