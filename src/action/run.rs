use crate::{config::ToolChain, error::Error, input::BuildSwitches, log_info_ln};
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::{
    path::PathBuf,
    process::{ExitCode, ExitStatus},
};

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
    let status = std::process::Command::new(PathBuf::from(".").join(&outfile))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .map_err(|_| Error::InvalidExe(outfile.clone()))?;

    graceful_crash(outfile, status)
}

#[cfg(windows)]
pub(super) fn graceful_crash(outfile: PathBuf, status: ExitStatus) -> Result<ExitCode, Error> {
    let code = status.code().unwrap() as u32;

    let sig_str = match code {
        0xC000_0005 => "STATUS_ACCESS_VIOLATION",
        0xC000_0017 => "STATUS_NO_MEMORY",
        0xC000_001D => "STATUS_ILLEGAL_INSTRUCTION",
        0xC000_008E => "STATUS_FLOAT_DIVIDE_BY_ZERO",
        0xC000_0094 => "STATUS_INTEGER_DIVIDE_BY_ZERO",
        0xC000_0096 => "STATUS_PRIVILEGED_INSTRUCTION",
        0xC000_00FD => "STATUS_STACK_OVERFLOW",
        0xC000_013A => "STATUS_CONTROL_C_EXIT",
        0xC000_0409 => "STATUS_STACK_BUFFER_OVERRUN",
        _ => {
            let code: u8 = code.try_into().unwrap_or(1);
            return Ok(code.into());
        }
    };
    Err(Error::ExeKilled(outfile, sig_str.to_string()))
}

#[cfg(unix)]
pub(super) fn graceful_crash(outfile: PathBuf, status: ExitStatus) -> Result<ExitCode, Error> {
    if let Some(sig) = status.signal() {
        let sig_str = match sig {
            3 => "SIGQUIT",
            4 => "SIGILL",
            5 => "SIGTRAP",
            6 => "SIGABRT",
            7 => "SIGBUS",
            8 => "SIGFPE",
            11 => "SIGSEGV",
            _ => "UNKNOWN SIGNAL",
        };
        if status.core_dumped() {
            return Err(Error::ExeKilled(outfile, format!("{sig_str}, core dumped")));
        } else {
            return Err(Error::ExeKilled(outfile, sig_str.to_string()));
        }
    }

    let code: u8 = status.code().unwrap().try_into().unwrap_or(1);
    Ok(code.into())
}
