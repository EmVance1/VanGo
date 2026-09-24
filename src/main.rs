mod action;
mod cli;
mod config;
mod deps;
mod error;
mod exec;

use error::Error;
use std::process::ExitCode;

macro_rules! exit_failure {
    ($($arg:tt)*) => { {
        log_error_ln!($($arg)*);
        std::process::exit(1);
    } };
}

fn read_manifest() -> Result<String, Error> {
    let os = if cfg!(windows) {
        ("win.Vango.toml", "mac.vango.toml")
    } else if cfg!(target_os = "linux") {
        ("lnx.Vango.toml", "mac.vango.toml")
    } else if cfg!(target_os = "macos") {
        ("mac.Vango.toml", "mac.vango.toml")
    } else {
        ("Vango.toml", "vango.toml")
    };

    if std::fs::exists(os.0).unwrap() {
        Ok(std::fs::read_to_string(os.0)?)
    } else if std::fs::exists(os.1).unwrap() {
        Ok(std::fs::read_to_string(os.1)?)
    } else if std::fs::exists("Vango.toml").unwrap() {
        Ok(std::fs::read_to_string("Vango.toml")?)
    } else if std::fs::exists("vango.toml").unwrap() {
        Ok(std::fs::read_to_string("vango.toml")?)
    } else {
        Err(Error::MissingBuildScript(
            std::env::current_dir().unwrap().file_name().unwrap().into(),
        ))
    }
}

fn main() -> ExitCode {
    let cmd = cli::collect_args();

    match cmd {
        cli::Action::Version => action::version(),
        cli::Action::Help { action } => action::help(action.as_deref()),
        cli::Action::New { name, opts } => {
            std::fs::create_dir(&name).unwrap_or_else(|e| exit_failure!("{}", e));
            std::env::set_current_dir(&name).unwrap_or_else(|e| exit_failure!("{}", e));
            action::init(opts).unwrap_or_else(|e| exit_failure!("{}", e));
        }
        cli::Action::Init { opts } => action::init(opts).unwrap_or_else(|e| exit_failure!("{}", e)),
        _ => {
            let manifest_src = read_manifest().unwrap_or_else(|e| exit_failure!("{}", e));
            let manifest = config::VangoFile::from_str(&manifest_src)
                .unwrap_or_else(|e| exit_failure!("{}", e))
                .get_build()
                .unwrap_or_else(|| exit_failure!("action requires source code ([package]) type manifest"));

            match cmd {
                cli::Action::Build { build } => {
                    let switches = build.into_switches(true);
                    action::build(&manifest, &switches, 1).unwrap_or_else(|e| exit_failure!("{}", e));
                }
                cli::Action::Run { build, args } => {
                    if manifest.artefact.is_lib() {
                        exit_failure!("{}", Error::LibNotExe(manifest.name));
                    }
                    let switches = build.into_switches(true);
                    action::build(&manifest, &switches, 1).unwrap_or_else(|e| exit_failure!("{}", e));
                    return action::run(&manifest, &switches, args).unwrap_or_else(|e| exit_failure!("{}", e));
                }
                cli::Action::Test { build, args } => {
                    let switches = build.into_switches(true);
                    action::build(&manifest, &switches, 0).unwrap_or_else(|e| exit_failure!("{}", e));
                    return action::test(manifest, &switches, args).unwrap_or_else(|e| exit_failure!("{}", e));
                }
                cli::Action::Clean => {
                    action::clean(&manifest).unwrap_or_else(|e| exit_failure!("{}", e));
                }
                cli::Action::Gen {
                    #[allow(unused)]
                    target,
                } => {
                    action::clangd(&manifest, false).unwrap_or_else(|e| exit_failure!("{}", e));
                }
                _ => unreachable!(),
            }
        }
    }

    0.into()
}
