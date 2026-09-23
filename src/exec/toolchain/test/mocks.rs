use super::*;
use crate::{config::{BuildSettings, PrecompiledHeader, Runtime, WarnLevel}, exec::BuildInfo};

impl BuildInfo {
    fn mock_base(outfile: &Path) -> Self {
        Self {
            artefact: Artefact::Executable,
            toolchain: Toolchain::Msvc,
            lang: Language::Cpp(120),
            cpprt: false,
            changed: false,
            is_testexe: false,
            settings: BuildSettings {
                opt_level: 0,
                opt_size: false,
                opt_speed: false,
                opt_linktime: false,
                iso_compliant: false,
                warn_level: WarnLevel::Basic,
                warn_as_error: false,
                debug_info: true,
                runtime: Runtime::DynamicDebug,
                aslr: true,
                no_rtti: false,
                no_except: false,

                pthreads: false,
                asan: false,
                tsan: false,
                lsan: false,
                ubsan: false,
            },
            defines: vec![],
            srcdir: "src".into(),
            incdirs: vec!["src".into()],
            libdirs: vec![],
            rpaths: vec![],
            outdir: "bin\\debug".into(),
            pch: vec![],
            sources: vec![],
            headers: vec![],
            archives: vec![],
            relink: vec![],
            outfile: outfile.to_owned(),
            implib: None,

            comp_args: vec![],
            link_args: vec![],
        }
    }

    pub fn mock_debug(
        outfile: &Path,
        artefact: Artefact,
        lang: Language,
        toolchain: Toolchain,
        pch: Vec<PrecompiledHeader>,
        crtstatic: bool,
    ) -> Self {
        let base = Self::mock_base(outfile);
        let defines = if toolchain.is_windows() {
            vec!["UNICODE".to_string(), "_UNICODE".to_string()]
        } else {
            vec![]
        };

        Self {
            artefact,
            toolchain,
            lang,
            pch,
            settings: BuildSettings {
                runtime: if crtstatic { Runtime::StaticDebug } else { Runtime::DynamicDebug },
                ..base.settings
            },
            defines,
            ..base
        }
    }
    pub fn mock_release(
        outfile: &Path,
        artefact: Artefact,
        lang: Language,
        toolchain: Toolchain,
        pch: Vec<PrecompiledHeader>,
        crtstatic: bool,
    ) -> Self {
        let base = Self::mock_base(outfile);
        let defines = if toolchain.is_windows() {
            vec!["UNICODE".to_string(), "_UNICODE".to_string()]
        } else {
            vec![]
        };

        Self {
            artefact,
            toolchain,
            lang,
            pch,
            settings: BuildSettings {
                opt_level: 3,
                opt_linktime: true,
                debug_info: false,
                runtime: if crtstatic {
                    Runtime::StaticRelease
                } else {
                    Runtime::DynamicRelease
                },
                ..base.settings
            },
            defines,
            ..base
        }
    }
}
