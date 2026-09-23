use super::*;

impl BuildProfile {
    pub fn default_debug() -> Self {
        Self {
            baseprof: Profile::Debug,
            defines: vec!["VANGO_DEBUG".to_string()],
            include: vec!["src".into()],
            pch: None,
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
            compiler_options: vec![],
            linker_options: vec![],
        }
    }

    pub fn default_release() -> Self {
        let mut base = Self::default_debug();
        base.baseprof = Profile::Release;
        base.defines = vec!["VANGO_RELEASE".to_string()];
        base.settings.opt_level = 3;
        base.settings.opt_linktime = true;
        base.settings.debug_info = false;
        base.settings.runtime = Runtime::DynamicRelease;
        base
    }

    #[allow(dead_code)]
    pub fn default_test() -> Self {
        let mut base = Self::default_debug();
        base.defines.push("VANGO_TEST".to_string());
        base
    }

    pub fn layer(mut self, other: raw::BuildProfile) -> Self {
        if let Some(base) = other.inherits {
            if base == "debug" {
                self.baseprof = Profile::Debug;
            } else {
                self.baseprof = Profile::Release;
            }
        }

        self.defines.extend(other.defines);
        self.include.extend(other.include);
        self.pch = other.pch;

        other.build_settings.opt_level.inspect(|v| self.settings.opt_level = *v);
        other.build_settings.opt_size.inspect(|v| self.settings.opt_size = *v);
        other.build_settings.opt_speed.inspect(|v| self.settings.opt_speed = *v);
        other.build_settings.opt_linktime.inspect(|v| self.settings.opt_linktime = *v);
        other.build_settings.iso_compliant.inspect(|v| self.settings.iso_compliant = *v);
        other.build_settings.warn_level.inspect(|v| self.settings.warn_level = *v);
        other.build_settings.warn_as_error.inspect(|v| self.settings.warn_as_error = *v);
        other.build_settings.debug_info.inspect(|v| self.settings.debug_info = *v);
        other.build_settings.runtime.inspect(|v| self.settings.runtime = *v);

        other.build_settings.aslr.inspect(|v| self.settings.aslr = *v);
        other.build_settings.no_rtti.inspect(|v| self.settings.no_rtti = *v);
        other.build_settings.no_except.inspect(|v| self.settings.no_except = *v);
        other.build_settings.pthreads.inspect(|v| self.settings.pthreads = *v);

        other.build_settings.sanitize.address.inspect(|v| self.settings.asan = *v);
        other.build_settings.sanitize.thread.inspect(|v| self.settings.tsan = *v);
        other.build_settings.sanitize.leak.inspect(|v| self.settings.lsan = *v);
        other.build_settings.sanitize.undefined.inspect(|v| self.settings.ubsan = *v);

        // self.settings.cpprt = self.settings.runtime.as_ref().map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default()

        self.compiler_options.extend(other.compiler_options);
        self.linker_options.extend(other.linker_options);

        self
    }
}

