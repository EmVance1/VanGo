mod mocks;

use super::*;
use crate::config::{Artefact, Language};

#[test]
pub fn compile_cmd_gcc_dbg1() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.o");

    let cmd = Toolchain::Gcc.compiler().command(
        &src,
        &obj,
        &BuildInfo::mock_debug(&out, Artefact::Executable, Language::Cpp(20), Toolchain::Gcc, vec![], false),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "-H",
            "-std=c++20",
            "-fpie",
            "-c",
            "-O0",
            "-g",
            "-Wall",
            "-Isrc",
            src.to_str().unwrap(),
            &format!("-o{}", obj.display())
        ]
    );
}

#[test]
pub fn compile_cmd_clang_mingw_dbg1() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.o");

    let cmd = Toolchain::ClangMingw.compiler().command(
        &src,
        &obj,
        &BuildInfo::mock_debug(&out, Artefact::Executable, Language::Cpp(23), Toolchain::ClangMingw, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "--target=x86_64-w64-mingw32",
            "-H",
            "-std=c++23",
            "-c",
            "-O0",
            "-g",
            "-Wall",
            "-Isrc",
            "-DUNICODE",
            "-D_UNICODE",
            src.to_str().unwrap(),
            &format!("-o{}", obj.display())
        ]
    );
}

#[test]
pub fn compile_cmd_clang_gcc_dbg1() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.o");

    let cmd = Toolchain::ClangGcc.compiler().command(
        &src,
        &obj,
        &BuildInfo::mock_debug(&out, Artefact::Executable, Language::Cpp(23), Toolchain::ClangGcc, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "-H",
            "-std=c++23",
            "-fpie",
            "-c",
            "-O0",
            "-g",
            "-Wall",
            "-Isrc",
            src.to_str().unwrap(),
            &format!("-o{}", obj.display())
        ]
    );
}

#[test]
pub fn compile_cmd_gcc_rel1() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/release");
    let obj = Path::new("bin/release/obj/main.o");

    let cmd = Toolchain::Gcc.compiler().command(
        &src,
        &obj,
        &BuildInfo::mock_release(&out, Artefact::Executable, Language::Cpp(20), Toolchain::Gcc, vec![], false),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    if cfg!(windows) {
        assert_eq!(
            cmd,
            [
                "-H",
                "-std=c++20",
                "-c",
                "-O3",
                "-flto",
                "-Wall",
                "-Isrc",
                "-DUNICODE",
                "-D_UNICODE",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display())
            ]
        );
    } else {
        assert_eq!(
            cmd,
            [
                "-H",
                "-std=c++20",
                "-fpie",
                "-c",
                "-O3",
                "-flto",
                "-Wall",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display())
            ]
        );
    }
}

#[test]
pub fn compile_cmd_gcc_rel2() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/release");
    let obj = Path::new("bin/release/obj/main.o");

    let cmd = Toolchain::Gcc.compiler().command(
        &src,
        &obj,
        &BuildInfo::mock_release(&out, Artefact::Executable, Language::Cpp(23), Toolchain::Gcc, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    if cfg!(windows) {
        assert_eq!(
            cmd,
            [
                "-H",
                "-std=c++23",
                "-c",
                "-O3",
                "-flto",
                "-Wall",
                "-Isrc",
                "-DUNICODE",
                "-D_UNICODE",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display())
            ]
        );
    } else {
        assert_eq!(
            cmd,
            [
                "-H",
                "-std=c++23",
                "-fpie",
                "-c",
                "-O3",
                "-flto",
                "-Wall",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display())
            ]
        );
    }
}

#[test]
pub fn compile_cmd_gcc_staticlib() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/release");
    let obj = Path::new("bin/release/obj/main.o");

    let cmd = Toolchain::Gcc.compiler().command(
        &src,
        &obj,
        &BuildInfo::mock_debug(&out, Artefact::StaticLib, Language::Cpp(20), Toolchain::Gcc, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    if cfg!(windows) {
        assert_eq!(
            cmd,
            [
                "-H",
                "-std=c++20",
                "-c",
                "-O0",
                "-g",
                "-Wall",
                "-Isrc",
                "-DUNICODE",
                "-D_UNICODE",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display())
            ]
        );
    } else {
        assert_eq!(
            cmd,
            [
                "-H",
                "-std=c++20",
                "-fPIC",
                "-c",
                "-O0",
                "-g",
                "-Wall",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display())
            ]
        );
    }
}

#[test]
#[cfg(windows)]
pub fn compile_cmd_mingw_sharedlib() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/release");
    let obj = Path::new("bin/release/obj/main.o");

    let cmd = Toolchain::Mingw.compiler().command(
        src,
        obj,
        &BuildInfo::mock_debug(out, Artefact::SharedLib, Language::Cpp(20), Toolchain::Mingw, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "-H",
            "-std=c++20",
            "-c",
            "-O0",
            "-g",
            "-Wall",
            "-Isrc",
            "-DUNICODE",
            "-D_UNICODE",
            src.to_str().unwrap(),
            &format!("-o{}", obj.display())
        ]
    );
}

#[test]
pub fn compile_cmd_msvc_dbg() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.obj");

    let cmd = Toolchain::Msvc.compiler().command(
        src,
        obj,
        &BuildInfo::mock_debug(out, Artefact::Executable, Language::Cpp(120), Toolchain::Msvc, vec![], false),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "/nologo",
            "/showIncludes",
            "/diagnostics:caret",
            "/c",
            "/std:c++20",
            "/Zc:__cplusplus",
            "/MDd",
            "/Od",
            "/Zi",
            "/FS",
            "/sdl",
            "/Fd:bin\\debug\\",
            "/Zf",
            "/W1",
            "/EHsc",
            "/Isrc",
            "/DUNICODE",
            "/D_UNICODE",
            src.to_str().unwrap(),
            &format!("/Fo:{}", obj.to_str().unwrap()),
        ]
    );
}

#[test]
pub fn compile_cmd_msvc_dbg2() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.obj");

    let cmd = Toolchain::ClangMsvc.compiler().command(
        src,
        obj,
        &BuildInfo::mock_debug(out, Artefact::Executable, Language::Cpp(123), Toolchain::ClangMsvc, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "/nologo",
            "/showIncludes",
            "/diagnostics:caret",
            "/c",
            "/std:c++latest",
            "/Zc:__cplusplus",
            "/MTd",
            "/Od",
            "/Zi",
            "/FS",
            "/sdl",
            "/Fd:bin\\debug\\",
            "/W1",
            "/EHsc",
            "/Isrc",
            "/DUNICODE",
            "/D_UNICODE",
            src.to_str().unwrap(),
            &format!("/Fo:{}", obj.to_str().unwrap()),
        ]
    );
}

#[test]
pub fn compile_cmd_msvc_rel1() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.obj");

    let cmd = Toolchain::Msvc.compiler().command(
        src,
        obj,
        &BuildInfo::mock_release(out, Artefact::Executable, Language::Cpp(123), Toolchain::Msvc, vec![], false),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "/nologo",
            "/showIncludes",
            "/diagnostics:caret",
            "/c",
            "/std:c++latest",
            "/Zc:__cplusplus",
            "/MD",
            "/O2",
            "/Oi",
            "/GL",
            "/W1",
            "/EHsc",
            "/Isrc",
            "/DUNICODE",
            "/D_UNICODE",
            src.to_str().unwrap(),
            &format!("/Fo:{}", obj.to_str().unwrap()),
        ]
    );
}

#[test]
pub fn compile_cmd_msvc_rel2() {
    let src = Path::new("src/main.cpp");
    let out = Path::new("bin/debug");
    let obj = Path::new("bin/debug/obj/main.obj");

    let cmd = Toolchain::Msvc.compiler().command(
        src,
        obj,
        &BuildInfo::mock_release(out, Artefact::Executable, Language::Cpp(123), Toolchain::Msvc, vec![], true),
        &PreCompHead::None,
        false,
        false,
    );

    let cmd: Vec<_> = cmd.get_args().collect();
    assert_eq!(
        cmd,
        [
            "/nologo",
            "/showIncludes",
            "/diagnostics:caret",
            "/c",
            "/std:c++latest",
            "/Zc:__cplusplus",
            "/MT",
            "/O2",
            "/Oi",
            "/GL",
            "/W1",
            "/EHsc",
            "/Isrc",
            "/DUNICODE",
            "/D_UNICODE",
            src.to_str().unwrap(),
            &format!("/Fo:{}", obj.to_str().unwrap()),
        ]
    );
}
