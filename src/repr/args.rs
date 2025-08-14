use super::{Config, ToolChain, Lang};


pub struct Args(pub ToolChain);

#[allow(unused, non_snake_case)]
impl Args {
    pub fn no_link(&self) -> &'static str {
        if self.0.is_msvc() { "/c" } else { "-c" }
    }
    pub fn comp_output(&self, file: &str) -> String {
        if self.0.is_msvc() { format!("/Fo:{file}") } else { format!("-o{file}") }
    }
    pub fn link_output(&self, file: &str) -> String {
        if self.0.is_msvc() { format!("/OUT:{file}") } else { format!("-o{file}") }
    }
    pub fn eh_default_cpp(&self) -> Option<&'static str> {
        if self.0.is_msvc() { Some("/EHsc") } else { None }
    }
    pub fn dbg_symbols(&self) -> &'static str {
        if self.0.is_msvc() { "/Zi" } else { "-g" }
    }

    pub fn std(&self, lang: Lang) -> String {
        if self.0.is_msvc() {
            if lang.is_cpp() {
                if lang.is_latest() {
                    "/std:c++latest".to_string()
                } else {
                    format!("/std:c++{}", lang.numeric())
                }
            } else if lang.is_latest() {
                "/std:clatest".to_string()
            } else {
                format!("/std:c{}", lang.numeric())
            }
        } else if lang.is_cpp() {
            format!("-std=c++{}", lang.numeric())
        } else {
            format!("-std=c{}", lang.numeric())
        }
    }

    pub fn O0(&self) -> &'static str {
        if self.0.is_msvc() { "/Od" } else { "-O0" }
    }
    pub fn O1(&self) -> &'static str {
        if self.0.is_msvc() { "/O1" } else { "-O1" }
    }
    pub fn O2(&self) -> &'static str {
        if self.0.is_msvc() { "/O2" } else { "-O2" }
    }
    pub fn O3(&self) -> &'static str {
        if self.0.is_msvc() { "/O2" } else { "-O3" }
    }
    pub fn Os(&self) -> &'static str {
        if self.0.is_msvc() { "/Os" } else { "-Os" }
    }
    pub fn Ot(&self) -> &'static str {
        if self.0.is_msvc() { "/Ot" } else { "-Ofast" }
    }

    pub fn I(&self) -> &'static str {
        if self.0.is_msvc() { "/I" } else { "-I" }
    }
    pub fn D(&self) -> &'static str {
        if self.0.is_msvc() { "/D" } else { "-D" }
    }
    pub fn L(&self) -> &'static str {
        if self.0.is_msvc() { "/LIBPATH:" } else { "-L" }
    }
    pub fn l(&self) -> &'static str {
        if self.0.is_msvc() { "" } else { "-l" }
    }

    pub fn crt_static(&self, config: Config) -> &'static str {
        if self.0.is_msvc() { if config.is_release() { "/MT" } else { "/MTd" } } else { "-static" }
    }
    pub fn crt_dynamic(&self, config: Config) -> Option<&'static str> {
        if self.0.is_msvc() { if config.is_release() { Some("/MD") } else { Some("/MDd") } } else { None }
    }

    pub fn opt_profile_none(&self) -> Vec<String> {
        if self.0.is_msvc() {
            vec![ self.O0().to_string(), self.dbg_symbols().to_string(), "/Fd:bin/debug/obj/vc143.pdb".to_string(), "/FS".to_string() ]
        } else {
            vec![ self.O0().to_string(), self.dbg_symbols().to_string() ]
        }
    }
    pub fn opt_profile_high(&self) -> Vec<String> {
        if self.0.is_msvc() {
            vec![ self.O2().to_string(), "/Oi".to_string(), "/GL".to_string() ]
        } else {
            vec![ self.O2().to_string() ]
        }
    }
}

