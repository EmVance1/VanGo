use std::io::Write;
use crate::{
    config::VangoFile,
    error::Error,
    log_info_ln,
};
use super::generate;


pub fn new(library: bool, is_c: bool, clangd: bool, name: &str) -> Result<(), Error> {
    std::fs::create_dir(name)?;
    std::env::set_current_dir(name)?;
    init(library, is_c, clangd)
}


pub fn init(library: bool, is_c: bool, clangd: bool) -> Result<(), Error> {
    let name = std::env::current_dir().unwrap().file_name().unwrap().to_string_lossy().to_string();
    log_info_ln!("creating new {} project: {}", if library { "library" } else { "application" }, name);
    let ext =    if is_c { "c" } else { "cpp" };
    let lang =   if is_c { "c11" } else { "c++17" };
    let header = if is_c { "stdio.h" } else { "cstdio" };
    std::fs::create_dir("src")?;
    if library {
        std::fs::create_dir_all(format!("include/{name}"))?;
        std::fs::write(format!("include/{name}/lib.h"), if is_c {
            "#ifndef LIB_H\n#define LIB_H\n\nint func(int a, int b);\n\n#endif"
        } else {
            "#pragma once\n\nint func(int a, int b);\n"
        })?;
        let toml = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nlang = \"{lang}\"\nkind = \"staticlib\"\ninclude = [ \"src\", \"include/{name}\" ]\ninclude-pub = \"include\"\n\n[dependencies]\n");
        std::fs::write(format!("src/lib.{ext}"), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n")?;
        std::fs::write("Vango.toml", &toml)?;
        if clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            generate(&build.unwrap_build())?;
        }
    } else {
        let toml = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nlang = \"{lang}\"\n\n[dependencies]\n");
        std::fs::write(format!("src/main.{ext}"), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n"))?;
        std::fs::write("Vango.toml", &toml)?;
        if clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            generate(&build.unwrap_build())?;
        }
    }
    log_info_ln!("successfully created project '{name}'");
    Ok(())
}

