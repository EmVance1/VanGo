use crate::{
    config::{Artefact, Language, Runtime, WarnLevel},
    exec::{BuildInfo, PreCompHead},
};
use std::path::{Path, PathBuf};

pub fn compiler_args(cmd: &mut std::process::Command, src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, _verbose: bool) {
    cmd.args(&info.comp_args);
    cmd.arg("/nologo"); // output configuration (see output parser)
    cmd.arg("/showIncludes"); // "
    cmd.arg("/diagnostics:caret"); // "
    // cmd.arg("/WL");              // ", one line diagnostics
    cmd.arg("/c");
    match info.lang {
        Language::Cpp(123) => {
            cmd.arg("/std:c++latest");
        }
        Language::Cpp(n) if n < 114 => {
            cmd.arg("/std:c++14");
        }
        Language::C(120) => {
            cmd.arg("/std:clatest");
        }
        Language::C(99) => {} // extensions on by default
        Language::C(89) => {
            cmd.arg("/Za"); // disable MS pseudo C99 extensions
        }
        Language::Cpp(_) | Language::C(_) => {
            cmd.arg(format!("/std:{}", info.lang));
        }
    }
    if info.lang.is_cpp() {
        cmd.arg("/Zc:__cplusplus"); // correctly define '__cplusplus' macro
    } else {
        cmd.arg("/TC"); // enforce C for all sources (needed for MS pseudo C99)
    }
    match info.settings.runtime {
        Runtime::DynamicDebug => {
            if info.settings.asan && info.toolchain.is_llvm() {
                cmd.arg("/MD");
            } else {
                cmd.arg("/MDd");
            }
        }
        Runtime::DynamicRelease => {
            cmd.arg("/MD");
        }
        Runtime::StaticDebug => {
            if info.settings.asan && info.toolchain.is_llvm() {
                cmd.arg("/MT");
            } else {
                cmd.arg("/MTd");
            }
        }
        Runtime::StaticRelease => {
            cmd.arg("/MT");
        }
    }
    match info.settings.opt_level {
        0 => {
            cmd.arg("/Od");
        }
        1 => {
            cmd.arg("/Ox");
        }
        2 => {
            cmd.arg("/O1");
        }
        3 => {
            cmd.args(["/O2", "/Oi"]);
        }
        _ => (),
    }
    if info.settings.opt_size {
        cmd.arg("/Os");
    }
    if info.settings.opt_speed {
        cmd.arg("/Ot");
    }
    if info.settings.opt_linktime && !info.is_testexe {
        cmd.arg("/GL");
    }
    if info.settings.debug_info {
        cmd.args(["/Zi", "/FS", "/sdl"]); // debug info, thread safe, extra security
        cmd.arg(format!("/Fd:{}\\", info.outdir.display())); // PDB output dir
        if !info.toolchain.is_llvm() {
            cmd.arg("/Zf");
        } // faster PDB gen??
    }
    match info.settings.warn_level {
        WarnLevel::None => {
            cmd.arg("/w");
        }
        WarnLevel::Basic => {
            cmd.arg("/W1");
        }
        WarnLevel::High => {
            cmd.arg("/W4");
        }
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    if info.settings.iso_compliant {
        cmd.arg("/permissive-");
    }
    if info.lang.is_cpp() {
        if info.settings.no_rtti {
            cmd.arg("/GR-"); // rtti on by default
        }
        if info.settings.no_except {
            cmd.arg("/EHsc-");
        } else {
            cmd.arg("/EHsc"); // default C++ exception handling, extern "C" -> noexcept
        }
    }
    if info.settings.asan {
        cmd.arg("-fsanitize=address");
    }
    // most sanitizers not supported by MSVC
    /*
    if info.settings.tsan {
        cmd.arg("-fsanitize=thread");
    }
    if info.settings.lsan {
        cmd.arg("-fsanitize=leak");
    }
    */
    if info.settings.ubsan && info.toolchain.is_llvm() {
        cmd.arg("-fsanitize=undefined");
    }
    cmd.args(info.incdirs.iter().map(|inc| format!("/I{}", inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("/D{def}")));
    match pch {
        PreCompHead::Create(h) => {
            cmd.arg(format!("/Yc{}", h.display()));
            cmd.arg(format!("/Fp:{}", info.outdir.join("pch").join(h).with_extension("h.pch").display()));
        }
        PreCompHead::Use(h) => {
            cmd.arg(format!("/Yu{}", h.display()));
            cmd.arg(format!("/Fp:{}", info.outdir.join("pch").join(h).with_extension("h.pch").display()));
        }
        PreCompHead::None => (),
    }

    cmd.arg(src);
    cmd.arg(format!("/Fo:{}", obj.display()));

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
}

pub fn linker_args(cmd: &mut std::process::Command, objs: Vec<PathBuf>, info: BuildInfo, _verbose: bool) {
    cmd.args(info.link_args);
    cmd.arg("/NOLOGO");
    cmd.arg("/MACHINE:X64");
    if let Artefact::SharedLib = info.artefact {
        cmd.arg("/DLL");
        if let Some(implib) = info.implib {
            cmd.arg(format!("/IMPLIB:{}", implib.display()));
        }
    }
    if info.settings.aslr {
        cmd.arg("/DYNAMICBASE");
    }
    if info.settings.debug_info {
        cmd.arg("/DEBUG");
    }
    if info.settings.opt_linktime && !info.is_testexe {
        cmd.arg("/LTCG"); // link-time codegen, iff /GL
        cmd.arg("/OPT:REF"); // strip unreferenced symbols
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    cmd.args(objs);
    cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l.display())));
    cmd.args(info.archives);
    if info.settings.asan && info.toolchain.is_llvm() {
        cmd.arg("clang_rt.asan_dynamic-x86_64.lib");
        cmd.arg("clang_rt.asan_dynamic_runtime_thunk-x86_64.lib");
    }
    cmd.args(DEFAULT_LIBS);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));
}

pub fn archiver_args(cmd: &mut std::process::Command, objs: Vec<PathBuf>, info: BuildInfo, _verbose: bool) {
    cmd.args(info.link_args);
    cmd.arg("/NOLOGO");
    cmd.arg("/MACHINE:X64");
    if info.settings.opt_linktime {
        cmd.arg("/LTCG"); // link-time codegen HINT, iff /GL
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    cmd.args(objs);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));
}

const DEFAULT_LIBS: &[&str] = &[
    "kernel32.lib",
    "user32.lib",
    "winspool.lib",
    "comdlg32.lib",
    "advapi32.lib",
    "shell32.lib",
    "ole32.lib",
    "oleaut32.lib",
    "uuid.lib",
    "odbc32.lib",
    "odbccp32.lib",
    "gdi32.lib",
];
