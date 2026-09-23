use crate::{log_error_ln, log_warn_ln};
use std::{io::BufRead, path::PathBuf};

pub fn compiler(output: &std::process::Output) -> bool {
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.contains(" warning D") {
            log_warn_ln!("{line}");
        } else {
            log_error_ln!("{line}");
        }
    }
    let mut includes = vec![];
    for line in output.stdout.lines().skip(1) {
        let line = line.unwrap();
        if let Some(inc) = line.strip_prefix("Note: including file:") {
            let inc = inc.trim();
            if !inc.starts_with("C:\\Program Files") {
                includes.push(PathBuf::from(inc));
            }
        } else if line.contains(": error C") || line.contains(": fatal error C") {
            log_error_ln!("{line}");
        } else if line.contains(": warning C") {
            log_warn_ln!("{line}");
        } else {
            println!("{line}");
        }
    }
    // println!("{:?}", includes);
    output.status.success()
}

pub fn linker(output: &std::process::Output, clang: bool) -> bool {
    if clang {
        for line in output.stderr.lines() {
            let line = line.unwrap();
            if line.contains("lld-link: error") {
                log_error_ln!("{line}");
            } else if line.contains("lld-link: warning:") {
                log_warn_ln!("{line}");
            } else {
                println!("{line}");
            }
        }
    } else {
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
    }
    output.status.success()
}

pub fn archiver(output: &std::process::Output, clang: bool) -> bool {
    if clang {
        for line in output.stderr.lines() {
            let line = line.unwrap();
            log_warn_ln!("{line}");
        }
    } else {
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
    }
    output.status.success()
}
