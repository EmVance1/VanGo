use super::*;
use crate::config::{WarnLevel, Runtime};


impl BuildInfo {
    pub fn mock_base(outfile: &Path) -> Self {
        let defines = if cfg!(windows) {
            vec![ "UNICODE".to_string(), "_UNICODE".to_string() ]
        } else {
            vec![]
        };

        Self {
            projkind: ProjKind::App,
            toolchain: ToolChain::Msvc,
            lang: Lang::Cpp(120),
            crtstatic: false,
            cpprt: false,
            changed: false,
            settings: BuildSettings{
                opt_level: 0,
                opt_size: false,
                opt_speed: false,
                opt_linktime: false,
                iso_compliant: false,
                warn_level: WarnLevel::Basic,
                warn_as_error: false,
                debug_info: true,
                runtime: Runtime::DynamicDebug,
                pthreads: false,
                aslr: true,
                rtti: true,
            },
            defines,
            srcdir:   "src".into(),
            incdirs:  vec![ "src".into() ],
            libdirs:  vec![],
            outdir:   "bin".into(),
            pch:      None,
            sources:  vec![],
            headers:  vec![],
            archives: vec![],
            relink:   vec![],
            outfile:  outfile.to_owned(),
            implib:   None,

            comp_args: vec![],
            link_args: vec![],
        }
    }

    pub fn mock_debug(outfile: &Path, projkind: ProjKind, lang: Lang, toolchain: ToolChain, pch: Option<PathBuf>, crtstatic: bool) -> Self {
        let base = Self::mock_base(outfile);
        Self {
            projkind,
            toolchain,
            lang,
            pch,
            settings: BuildSettings{
                runtime: if crtstatic { Runtime::StaticDebug } else { Runtime::DynamicDebug },
                ..base.settings
            },
            ..base
        }
    }
    pub fn mock_release(outfile: &Path, projkind: ProjKind, lang: Lang, toolchain: ToolChain, pch: Option<PathBuf>, crtstatic: bool) -> Self {
        let base = Self::mock_base(outfile);
        Self {
            projkind,
            toolchain,
            lang,
            pch,
            settings: BuildSettings{
                opt_level: 3,
                opt_linktime: true,
                debug_info: false,
                runtime: if crtstatic { Runtime::StaticRelease } else { Runtime::DynamicRelease },
                ..base.settings
            },
            ..base
        }
    }
}

