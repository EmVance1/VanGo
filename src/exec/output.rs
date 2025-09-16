use std::{io::BufRead, path::PathBuf};
use crate::{log_error_ln, log_warn_ln};


pub fn msvc_compiler(output: &std::process::Output) -> bool {
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


fn gnu_is_sys_include(path: &str) -> bool {
    if cfg!(windows) {
        path.starts_with("C:/msys64")
    } else {
        path.starts_with("/usr/include") || path.starts_with("/usr/lib")
    }
}

pub fn gnu_compiler(output: &std::process::Output) -> bool {
    let mut includes = vec![];
    let mut skip_until = false;
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.contains("In function") {
            continue
        }
        if line.starts_with('.') {
            let inc = line.trim_start_matches('.').trim();
            if !gnu_is_sys_include(inc) {
                includes.push(PathBuf::from(inc));
            }
        } else if line.contains(": error: ") || line.contains(": fatal error: ") {
            skip_until = false;
            log_error_ln!("{line}");
        } else if line.contains(" warning: ") {
            skip_until = false;
            log_warn_ln!("{line}");
        } else if line == "Multiple include guards may be useful for:" {
            skip_until = true;
        } else if !gnu_is_sys_include(&line) && !skip_until {
            println!("{line}");
        }
    }
    // println!("{:?}", includes);
    output.status.success()
}


pub fn msvc_linker(output: &std::process::Output, clang: bool) -> bool {
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

pub fn gnu_linker(output: &std::process::Output) -> bool {
    for line in output.stderr.lines() {
        let line = line.unwrap();
        if line.starts_with("collect2.exe") || line.contains("linker command failed with exit code 1") {
            continue
        }
        if let Some((_, err)) = line.split_once("ld.exe: ") {
            log_error_ln!("ld.exe: {err}");
        } else {
            log_error_ln!("{line}");
        }
    }
    output.status.success()
}


pub fn msvc_archiver(output: &std::process::Output, clang: bool) -> bool {
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

pub fn gnu_archiver(output: &std::process::Output) -> bool {
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

