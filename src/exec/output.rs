use std::io::BufRead;
use super::*;
use crate::{log_error_ln, log_warn_ln};


pub fn msvc_compiler(output: std::process::Output) -> bool {
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.contains(" warning D") {
            log_warn_ln!("{line}");
        } else {
            log_error_ln!("{line}");
        }
    }
    for line in output.stdout.lines().skip(1) {
        let line = line.unwrap();
        if line.contains(": error C") || line.contains(": fatal error C") {
            log_error_ln!("{line}");
        } else if line.contains(": warning C") {
            log_warn_ln!("{line}");
        } else {
            println!("{line}");
        }
    }
    output.status.success()
}

pub fn gnu_compiler(output: std::process::Output) -> bool {
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


pub fn msvc_linker(output: std::process::Output) -> bool {
    for line in output.stdout.lines() {
        let line = line.unwrap();
        if line.contains(" : error LNK") || line.contains(" : fatal error LNK") {
            log_error_ln!("{line}");
        } else if line.contains(" : warning LNK") {
            log_warn_ln!("{line}");
        } else if !line.contains("enerating code") {
            println!("{line}");
        }
    }
    output.status.success()
}

pub fn gnu_linker(output: std::process::Output) -> bool {
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


pub fn msvc_archiver(output: std::process::Output) -> bool {
    for line in output.stdout.lines() {
        let line = line.unwrap();
        if line.contains(" : error LNK") || line.contains(" : fatal error LNK") {
            log_error_ln!("{line}");
        } else if line.contains(" : warning LNK") {
            log_warn_ln!("{line}");
        } else if !line.contains("enerating code") {
            println!("{line}");
        }
    }
    output.status.success()
}

pub fn gnu_archiver(output: std::process::Output) -> bool {
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

