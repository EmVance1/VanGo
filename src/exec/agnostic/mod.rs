#![allow(unused)]
mod repr;

use std::{collections::HashMap, ffi::OsString, path::Path};
use crate::config::ProjKind;

use super::{CompileInfo, PreCompHead};


enum Setting {
    Opt0,
    Opt1,
    Opt2,
    Opt3,
    OptSize,
    OptSpeed,
    OptLinkTime,
    IsoCompliant,
    WarnNone,
    WarnBasic,
    WarnHigh,
    WarnAsError,
    DebugInfo,
    RuntimeStaticDebug,
    RuntimeStaticRelease,
    RuntimeDynamicDebug,
    RuntimeDynamicRelease,
    Aslr,
}

/*

pub fn compile_cmd(src: &Path, obj: &Path, info: CompileInfo, echo: bool, verbose: bool) -> std::process::Command {
    let tc = repr::Toolchain::default();

    let mut cmd = tc.compile(info.lang);
    cmd.args(info.comp_args);
    if info.lang.is_cpp() {
        cmd.args(tc.compiler.eh_default);
    }
    cmd.args(tc.compiler.standard);
    cmd.args(tc.compiler.comp_only);
    for set in info.profile.settings {
        match set {
            Setting::Opt0         => { cmd.args(tc.compiler.opt_0); }
            Setting::Opt1         => { cmd.args(tc.compiler.opt_1); }
            Setting::Opt2         => { cmd.args(tc.compiler.opt_2); }
            Setting::Opt3         => { cmd.args(tc.compiler.opt_3); }
            Setting::OptSize      => { cmd.args(tc.compiler.opt_size); }
            Setting::OptSpeed     => { cmd.args(tc.compiler.opt_speed); }
            Setting::OptLinkTime  => { cmd.args(tc.compiler.opt_linktime); }
            Setting::IsoCompliant => { cmd.args(tc.compiler.iso_compliant); }
            Setting::WarnNone     => { cmd.args(tc.compiler.warn_none); }
            Setting::WarnBasic    => { cmd.args(tc.compiler.warn_basic); }
            Setting::WarnHigh     => { cmd.args(tc.compiler.warn_high); }
            Setting::WarnAsError  => { cmd.args(tc.compiler.warn_as_error); }
            Setting::DebugInfo    => { cmd.args(tc.compiler.debug_info); }
            Setting::RuntimeStaticDebug    => { cmd.args(tc.compiler.runtime_static_debug); }
            Setting::RuntimeStaticRelease  => { cmd.args(tc.compiler.runtime_static_release); }
            Setting::RuntimeDynamicDebug   => { cmd.args(tc.compiler.runtime_dynamic_debug); }
            Setting::RuntimeDynamicRelease => { cmd.args(tc.compiler.runtime_dynamic_release); }
            Setting::Aslr => match info.projkind {
                ProjKind::App       => { cmd.args(tc.compiler.aslr_app); }
                ProjKind::SharedLib => { cmd.args(tc.compiler.aslr_lib); }
                _ => (),
            }
        };
    }
    for inc in info.incdirs { cmd.args(tc.compiler.include); }
    for def in info.defines { cmd.args(tc.compiler.define);  }

    cmd.arg(src);                   // per call swap
    cmd.args(tc.compiler.output);   // per call swap

    match info.pch {
        PreCompHead::Create(h) => {
            let mut ycarg = OsString::from("/Yc");
            ycarg.push(h);
            let mut fparg = OsString::from("/Fp:");
            fparg.push(info.outdir.join("pch").join(h).with_extension("h.pch"));
            cmd.arg(ycarg);
            cmd.arg(fparg);
        }
        PreCompHead::Use(h) => {
            let mut yuarg = OsString::from("/Yu");
            yuarg.push(h);
            cmd.arg(yuarg);
            let mut fparg = OsString::from("/Fp:");
            fparg.push(info.outdir.join("pch").join(h).with_extension("h.pch"));
            cmd.arg(fparg);
        }
        _ => ()
    }

    cmd.stdout(std::process::Stdio::piped());
    if verbose {
        cmd.stderr(std::process::Stdio::piped());
    } else {
        cmd.stderr(std::process::Stdio::null());
    };
    if echo { print_command(&cmd); }
    cmd
}



fn print_command(cmd: &std::process::Command) {
    print!("{} ", cmd.get_program().to_string_lossy());
    for arg in cmd.get_args() {
        print!("{} ", arg.to_string_lossy());
    }
    println!();
}

*/
