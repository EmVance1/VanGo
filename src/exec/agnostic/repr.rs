use crate::config::Lang;


#[derive(Debug, Default)]
pub struct Toolchain {
    pub toolchain: TcProperties,
    pub compiler:  CompilerArgs,
    pub linker:    LinkerArgs,
    pub archiver:  ArchiverArgs,
}

impl Toolchain {
    pub fn compile(&self, lang: Lang) -> std::process::Command {
        if lang.is_cpp() {
            let mut cmd = std::process::Command::new(&self.toolchain.compiler_cpp[0]);
            for arg in self.toolchain.compiler_cpp.iter().skip(1) {
                cmd.arg(arg);
            }
            cmd
        } else {
            let mut cmd = std::process::Command::new(&self.toolchain.compiler_c[0]);
            for arg in self.toolchain.compiler_c.iter().skip(1) {
                cmd.arg(arg);
            }
            cmd
        }
    }
}


#[derive(Debug, Default)]
pub struct TcProperties {
    pub name: String,
    pub compiler_c:   Vec<String>,
    pub compiler_cpp: Vec<String>,
    pub linker_c:     Vec<String>,
    pub linker_cpp:   Vec<String>,
    pub archiver_c:   Vec<String>,
    pub archiver_cpp: Vec<String>,
    pub exe_fmt:     String,
    pub obj_fmt:     String,
    pub archive_fmt: String,
}

#[derive(Debug, Default)]
pub struct CompilerArgs {
    pub input:  Vec<String>,
    pub output: Vec<String>,

    pub standard:   Vec<String>,
    pub comp_only:  Vec<String>,
    pub eh_default: Vec<String>,
    pub debug_info: Vec<String>,

    pub include: Vec<String>,
    pub define:  Vec<String>,

    pub opt_0:     Vec<String>,
    pub opt_1:     Vec<String>,
    pub opt_2:     Vec<String>,
    pub opt_3:     Vec<String>,
    pub opt_size:  Vec<String>,
    pub opt_speed: Vec<String>,
    pub opt_linktime: Vec<String>,

    pub pch_create:  Vec<String>,
    pub pch_include: Vec<String>,

    pub runtime_static_debug:    Vec<String>,
    pub runtime_static_release:  Vec<String>,
    pub runtime_dynamic_debug:   Vec<String>,
    pub runtime_dynamic_release: Vec<String>,

    pub iso_compliant: Vec<String>,

    pub warn_none:     Vec<String>,
    pub warn_basic:    Vec<String>,
    pub warn_high:     Vec<String>,
    pub warn_as_error: Vec<String>,
}

#[derive(Debug, Default)]
pub struct LinkerArgs {
    pub object:  Vec<String>,
    pub libdir:  Vec<String>,
    pub library: Vec<String>,
    pub output:  Vec<String>,

    pub opt_linktime: Vec<String>,

    pub default_libs: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ArchiverArgs {
    pub object: Vec<String>,
    pub output: Vec<String>,
}

