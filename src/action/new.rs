use super::clangd;
use crate::{config::VangoFile, error::Error, log_info_ln};

pub fn new(library: bool, strict: bool, is_c: bool, clangd: bool, name: &str) -> Result<(), Error> {
    std::fs::create_dir(name)?;
    std::env::set_current_dir(name)?;
    init(library, strict, is_c, clangd)
}

pub fn init(library: bool, strict: bool, is_c: bool, gen_clangd: bool) -> Result<(), Error> {
    let name = std::env::current_dir()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    log_info_ln!(
        "creating new {} project: {}",
        if library { "library" } else { "application" },
        name
    );
    let ext = if is_c { "c" } else { "cpp" };
    let lang = if is_c { "C11" } else { "C++17" };
    let header = if is_c { "stdio.h" } else { "cstdio" };
    let warns = if strict {
        "warn-level = \"high\"\niso-compliant = true\n"
    } else {
        ""
    };
    std::fs::create_dir("src")?;
    if library {
        std::fs::create_dir_all(format!("include/{name}"))?;
        std::fs::write(
            format!("include/{name}/lib.h"),
            if is_c {
                "\
#ifndef LIB_H
#define LIB_H

int func(int a, int b);

#endif"
            } else {
                "\
#pragma once

int func(int a, int b);
"
            },
        )?;
        let toml = format!(
            "\
[package]
name = \"{name}\"
version = \"0.1.0\"
lang = \"{lang}\"
kind = \"staticlib\"

{warns}
[dependencies]
"
        );
        std::fs::write(
            format!("src/lib.{ext}"),
            format!(
                "\
#include \"{name}/lib.h\"

int func(int a, int b) {{
    return a + b;
}}
"
            ),
        )?;
        std::fs::write("Vango.toml", &toml)?;
        if gen_clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            clangd(&build.unwrap_build(), true)?;
        }
    } else {
        let toml = format!(
            "\
[package]
name = \"{name}\"
version = \"0.1.0\"
lang = \"{lang}\"

{warns}
[dependencies]
"
        );
        std::fs::write(
            format!("src/main.{ext}"),
            format!(
                "\
#include <{header}>

int main() {{
    printf(\"Hello World!\\n\");
}}
"
            ),
        )?;
        std::fs::write("Vango.toml", &toml)?;
        if gen_clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            clangd(&build.unwrap_build(), true)?;
        }
    }
    log_info_ln!("successfully created project '{name}'");
    Ok(())
}
