use std::io::BufRead;
use super::*;
use crate::{log_error_ln, log_warn_ln};


pub fn on_msvc_compile_finish(output: std::process::Output) -> bool {
    for line in output.stderr.lines().skip(3) {
        let line = line.unwrap();
        log_error_ln!("{}", line);
    }
    for line in output.stdout.lines().skip(1) {
        let line = line.unwrap();
        if line.contains("): error C") || line.contains("): fatal error C") {
            log_error_ln!("{line}");
        } else if line.contains("): warning C") {
            log_warn_ln!("{line}");
        } else {
            println!("{line}");
        }
    }
    output.status.success()
}

pub fn on_gnu_compile_finish(output: std::process::Output) -> bool {
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.contains("In function") {
            continue
        }
        if line.contains(": error: ") || line.contains(": fatal error: ") {
            log_error_ln!("{line}");
        } else if line.contains(" warning: ") {
            log_warn_ln!("{line}");
        } else {
            println!("{line}");
        }
    }
    output.status.success()
}


pub fn on_msvc_link_finish(output: std::process::Output) -> bool {
    for line in output.stdout.lines().skip(3) {
        let line = line.unwrap();
        if line.starts_with("LINK : error LNK") || line.starts_with("LINK : fatal error LNK") {
            log_error_ln!("{line}");
        } else if line.starts_with("LINK : warning LNK") {
            log_warn_ln!("{line}");
        } else {
            println!("{line}");
        }
    }
    output.status.success()
}

pub fn on_gnu_link_finish(output: std::process::Output) -> bool {
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if !line.starts_with("collect2.exe") {
            if let Some((_, err)) = line.split_once("/ld.exe: ") {
                log_error_ln!("ld.exe: {err}");
            } else {
                log_error_ln!("{line}");
            }
        }
    }
    output.status.success()
}

