use crate::{log_error_ln, log_warn_ln};
use std::{io::BufRead, path::PathBuf};


fn is_sys_include(path: &str) -> bool {
    if cfg!(windows) {
        path.starts_with("C:/msys64")
    } else {
        path.starts_with("/usr/include") || path.starts_with("/usr/lib")
    }
}

pub fn compiler(output: &std::process::Output) -> bool {
    let mut includes = vec![];
    let mut skip_once = false;
    let mut skip_until = false;
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.contains("In function") {
            continue;
        }
        if line.starts_with('.') {
            let inc = line.trim_start_matches('.').trim();
            if !is_sys_include(inc) {
                includes.push(PathBuf::from(inc));
            }
        } else if line.starts_with("! ") {
            skip_once = true;
            continue;
        } else if line.contains(": error: ") || line.contains(": fatal error: ") {
            skip_until = false;
            log_error_ln!("{line}");
        } else if line.contains(" warning: ") {
            skip_until = false;
            log_warn_ln!("{line}");
        } else if line == "Multiple include guards may be useful for:" {
            skip_until = true;
        } else if !is_sys_include(&line) && !skip_once && !skip_until {
            println!("{line}");
        }
        skip_once = false;
    }
    // println!("{:?}", includes);
    output.status.success()
}

pub fn linker(output: &std::process::Output) -> bool {
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.starts_with("collect2.exe") || line.contains("linker command failed with exit code 1") {
            continue;
        }
        if let Some((_, err)) = line.split_once("ld.exe: ") {
            log_error_ln!("ld.exe: {err}");
        } else {
            log_error_ln!("{line}");
        }
    }
    output.status.success()
}

pub fn archiver(output: &std::process::Output) -> bool {
    for (i, line) in output.stderr.lines().enumerate() {
        let line = line.unwrap();
        if i == 0 {
            log_error_ln!("{line}");
        } else {
            println!("{line}");
        }
    }
    output.status.success()
}

