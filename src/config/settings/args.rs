use super::{Profile, ToolChain, Lang};


pub struct Args(pub ToolChain);

#[allow(unused, non_snake_case)]
impl Args {
    pub fn comp_only(&self) -> &'static str {
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
    pub fn debug_symbols(&self) -> Vec<String> {
        if self.0.is_msvc() {
            vec![ "/Zi".to_string(), "/Fd:bin\\debug\\obj\\".to_string(), "/FS".to_string() ]
        } else {
            vec![ "-g".to_string() ]
        }
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

    pub fn O0(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/Od" ] } else { vec![ "-O0" ] }
    }
    pub fn O1(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/O1" ] } else { vec![ "-O1" ] }
    }
    pub fn O2(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/O2" ] } else { vec![ "-O2" ] }
    }
    pub fn O3(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/O2", "/Oi" ] } else { vec![ "-O3" ] }
    }
    pub fn Os(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/Os" ] } else { vec![ "-Os" ] }
    }
    pub fn Ot(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/Ot" ] } else { vec![ "-Ofast" ] }
    }
    pub fn Olinktime(&self) -> Vec<&'static str> {
        if self.0.is_msvc() { vec![ "/GL" ] } else { vec![ "-flto" ]  }
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

    pub fn crt_static(&self, profile: &Profile) -> &'static str {
        if self.0.is_msvc() { if profile.is_release() { "/MT" } else { "/MTd" } } else { "-static" }
    }
    pub fn crt_dynamic(&self, profile: &Profile) -> Option<&'static str> {
        if self.0.is_msvc() { if profile.is_release() { Some("/MD") } else { Some("/MDd") } } else { None }
    }
}

