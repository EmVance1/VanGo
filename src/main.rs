mod error;
mod exec;
mod fetch;
mod input;
mod repr;
mod config;
mod testfw;
#[macro_use]
mod log;
mod action;

use error::Error;
use repr::*;
use std::{
    io::Write,
    process::ExitCode,
};


macro_rules! exit_failure {
    ($($arg:tt)*) => { {
        log_error!($($arg)*);
        std::process::exit(1);
    } };
}


fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let cmd = input::parse_input(args).unwrap_or_else(|e| exit_failure!("{}", e));

    if let input::Action::New { library, is_c, name } = &cmd {
        action::new(*library, *is_c, name).unwrap_or_else(|e| exit_failure!("{}", e));
        0.into()
    } else if let input::Action::Init{ library, is_c } = &cmd {
        action::init(*library, *is_c).unwrap_or_else(|e| exit_failure!("{}", e));
        0.into()
    } else if let input::Action::Help{ action } = &cmd {
        action::help(action.clone());
        0.into()

    } else {
        let bfile = if cfg!(target_os = "windows") && std::fs::exists("win.build.json").unwrap() {
            std::fs::read_to_string("win.build.json").unwrap()
        } else if cfg!(target_os = "linux") && std::fs::exists("lnx.build.json").unwrap() {
            std::fs::read_to_string("lnx.build.json").unwrap()
        } else if cfg!(target_os = "macos") && std::fs::exists("mac.build.json").unwrap() {
            std::fs::read_to_string("mac.build.json").unwrap()
        } else {
            std::fs::read_to_string("build.json")
                .map_err(|_| Error::MissingBuildScript(std::env::current_dir().unwrap().file_name().unwrap().into()))
                .unwrap_or_else(|e| exit_failure!("{}", e))
        };
        let build = BuildFile::from_str(&bfile)
            .unwrap_or_else(|e| exit_failure!("{}", e));

        match cmd {
            input::Action::Build{ switches } => {
                let (_rebuilt, _outfile) = action::build(build, switches, false).unwrap_or_else(|e| exit_failure!("{}", e));
                0.into()
            }
            input::Action::Run{ switches, args } => {
                let (_rebuilt, outfile) = action::build(build, switches, false).unwrap_or_else(|e| exit_failure!("{}", e));
                exec::run_app(&outfile, args).unwrap_or_else(|e| exit_failure!("{}", e)).into()
            }
            input::Action::Test{ switches, args } => {
                let (_rebuilt, _outfile) = action::build(build.clone(), switches.clone(), true).unwrap_or_else(|e| exit_failure!("{}", e));
                testfw::test_lib(build, switches, args).unwrap_or_else(|e| exit_failure!("{}", e));
                0.into()
            }
            input::Action::Clean => {
                action::clean(build).unwrap_or_else(|e| exit_failure!("{}", e));
                0.into()
            }
            _ => 0.into()
        }
    }
}

