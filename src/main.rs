mod error;
mod exec;
mod fetch;
mod input;
mod repr;
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
        } else if cfg!(target_os = "linux") && std::fs::exists("linux.build.json").unwrap() {
            std::fs::read_to_string("linux.build.json").unwrap()
        } else if cfg!(target_os = "macos") && std::fs::exists("macos.build.json").unwrap() {
            std::fs::read_to_string("macos.build.json").unwrap()
        } else {
            std::fs::read_to_string("build.json")
                .map_err(|_| Error::FileNotFound("build.json".to_string()))
                .unwrap_or_else(|e| exit_failure!("{}", e))
        };
        let build = BuildFile::from_str(&bfile)
            .map_err(Error::JsonParse)
            .unwrap_or_else(|e| exit_failure!("{}", e));

        match cmd {
            input::Action::Build{ config, toolchain, verbose } => {
                let build = build.finalise(config);
                let (rebuilt, _) = action::build(build, config, toolchain, verbose, false).unwrap_or_else(|e| exit_failure!("{}", e));
                if rebuilt { 8.into() } else { 0.into() }
            }
            input::Action::Run{ config, toolchain, verbose, args } => {
                let build = build.finalise(config);
                let (_, outfile) = action::build(build, config, toolchain, verbose, false).unwrap_or_else(|e| exit_failure!("{}", e));
                exec::run_app(&outfile, args).into()
            }
            input::Action::Test{ config, toolchain, verbose, args } => {
                let build = build.finalise(config);
                action::build(build.clone(), config, toolchain, verbose, true).unwrap_or_else(|e| exit_failure!("{}", e));
                testfw::test_lib(build, config, toolchain, verbose, args).unwrap_or_else(|e| exit_failure!("{}", e));
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

