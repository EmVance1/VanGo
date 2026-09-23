use super::*;


impl LibProfile {
    pub fn default_debug() -> Self {
        Self {
            include: "include".into(),
            libdir: "bin/debug".into(),
            binaries: vec![],
            defines: vec![ "VANGO_DEBUG".to_string() ],
        }
    }

    pub fn default_release() -> Self {
        Self {
            include: "include".into(),
            libdir: "bin/release".into(),
            binaries: vec![],
            defines: vec![ "VANGO_RELEASE".to_string() ],
        }
    }

    pub fn layer(mut self, other: raw::LibProfile) -> Self {
        if let Some(inc) = other.include {
            self.include = inc;
        }
        if let Some(dir) = other.libdir {
            self.libdir = dir;
        }
        self.binaries.extend(other.binaries);
        self.defines.extend(other.defines);
        self
    }
}

