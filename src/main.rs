mod error;
mod exec;
mod fetch;
mod input;
mod config;
mod testfw;
#[macro_use]
mod log;
mod action;

use error::Error;
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

    if let input::Action::Help{ action } = &cmd {
        action::help(action.clone());
    } else if let input::Action::New { library, is_c, clangd, name } = &cmd {
        action::new(*library, *is_c, *clangd, name).unwrap_or_else(|e| exit_failure!("{}", e));
    } else if let input::Action::Init{ library, is_c, clangd } = &cmd {
        action::init(*library, *is_c, *clangd).unwrap_or_else(|e| exit_failure!("{}", e));

    } else {
        let bfile = if cfg!(target_os = "windows") && std::fs::exists("win.vango.toml").unwrap() {
            std::fs::read_to_string("win.vango.toml").unwrap()
        } else if cfg!(target_os = "linux") && std::fs::exists("lnx.vango.toml").unwrap() {
            std::fs::read_to_string("lnx.vango.toml").unwrap()
        } else if cfg!(target_os = "macos") && std::fs::exists("mac.vango.json").unwrap() {
            std::fs::read_to_string("mac.vango.toml").unwrap()
        } else {
            std::fs::read_to_string("vango.toml")
                .map_err(|_| Error::MissingBuildScript(std::env::current_dir().unwrap().file_name().unwrap().into()))
                .unwrap_or_else(|e| exit_failure!("{}", e))
        };
        let build = config::VangoFile::from_str(&bfile)
            .unwrap_or_else(|e| exit_failure!("{}", e))
            .unwrap_build();

        match cmd {
            input::Action::Build{ switches } => {
                let (_rebuilt, _outfile) = action::build(build, switches, false).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            input::Action::Run{ switches, args } => {
                let (_rebuilt, outfile) = action::build(build, switches, false).unwrap_or_else(|e| exit_failure!("{}", e));
                return exec::run_app(&outfile, args).unwrap_or_else(|e| exit_failure!("{}", e)).into()
            }
            input::Action::Test{ switches, args } => {
                let (_rebuilt, _outfile) = action::build(build.clone(), switches.clone(), true).unwrap_or_else(|e| exit_failure!("{}", e));
                testfw::test_lib(build, switches, args).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            input::Action::Clean => {
                action::clean(build).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            input::Action::Gen{ target: _ } => {
                action::generate(build).unwrap_or_else(|e| exit_failure!("{}", e));
            }
            _ => {}
        }
    }

    0.into()
}

