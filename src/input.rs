use crate::{error::Error, config::{Profile, ToolChain}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    New { library: bool, is_c: bool, clangd: bool, name: String },
    Init{ library: bool, is_c: bool, clangd: bool },
    Clean,
    Clangd,
    #[allow(dead_code)]
    Gen  { target: String },
    Build{ switches: BuildSwitches },
    Run  { switches: BuildSwitches, args: Vec<String> },
    Test { switches: BuildSwitches, args: Vec<String> },
    Help { action: Option<String> },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildSwitches {
    pub profile: Profile,
    pub toolchain: ToolChain,
    pub install:   bool,
    pub echo:      bool,
    pub verbose:   bool,
    pub is_test:   bool,
}


pub fn collect_args() -> Result<Action, Error> {
    let mut args: Vec<_> = std::env::args().collect();
    // if let Some(first) = args.first() && first.as_str() == std::env::current_exe()?.as_os_str() {
        args.remove(0);
    // }
    if args.is_empty() { return Ok(Action::Help{ action: None }) }
    parse_args(args)
}

fn parse_args(mut args: Vec<String>) -> Result<Action, Error> {
    match args.remove(0).as_str() {
        "new" => {
            let library = args.remove_if(|s| *s == "--lib").is_some();
            let is_c    = args.remove_if(|s| *s == "--c").is_some();
            let clangd  = args.remove_if(|s| *s == "--clangd").is_some();
            if args.len() == 1 {
                Ok(Action::New{ library, is_c, clangd, name: args.remove(0) })
            } else {
                Err(Error::ExtraArgs("new".to_string(), args))
            }
        }
        "init" => {
            let library = args.remove_if(|s| *s == "--lib").is_some();
            let is_c    = args.remove_if(|s| *s == "--c").is_some();
            let clangd  = args.remove_if(|s| *s == "--clangd").is_some();
            if args.is_empty() {
                Ok(Action::Init{ library, is_c, clangd })
            } else {
                Err(Error::ExtraArgs("init".to_string(), args))
            }
        }
        "build" | "b" => {
            let debug     = args.remove_if(|s| *s == "--debug"   || *s == "-d").is_some();
            let release   = args.remove_if(|s| *s == "--release" || *s == "-r").is_some();
            let toolchain = parse_toolchain(args.remove_if(|s| s.starts_with("--toolchain=") || s.starts_with("-t=")))?;
            let install   = args.remove_if(|s| *s == "--install").is_some();
            let echo      = args.remove_if(|s| *s == "--echo").is_some();
            let verbose   = args.remove_if(|s| *s == "--verbose" || *s == "-v").is_some();
            let profile   = parse_profile(args.remove_if(|s| s.starts_with("--profile=")), debug, release)?;
            if args.is_empty() {
                Ok(Action::Build{ switches: BuildSwitches{ profile, toolchain, install, echo, verbose, is_test: false } })
            } else {
                Err(Error::ExtraArgs("build".to_string(), args))
            }
        }
        "run" | "r" => {
            let user_args = if let Some(i) = args.iter().position(|a| *a == "--") {
                let mut temp = args.split_off(i);
                temp.remove(0);
                temp
            } else {
                vec![]
            };
            let debug     = args.remove_if(|s| *s == "--debug"   || *s == "-d").is_some();
            let release   = args.remove_if(|s| *s == "--release" || *s == "-r").is_some();
            let toolchain = parse_toolchain(args.remove_if(|s| s.starts_with("--toolchain=") || s.starts_with("-t=")))?;
            let install   = args.remove_if(|s| *s == "--install").is_some();
            let echo      = args.remove_if(|s| *s == "--echo").is_some();
            let verbose   = args.remove_if(|s| *s == "--verbose" || *s == "-v").is_some();
            let profile = parse_profile(args.remove_if(|s| s.starts_with("--profile=")), debug, release)?;
            if args.is_empty() {
                Ok(Action::Run{ switches: BuildSwitches{ profile, toolchain, install, echo, verbose, is_test: false }, args: user_args })
            } else {
                Err(Error::ExtraArgs("run".to_string(), args))
            }
        }
        "test" | "t" => {
            let debug     = args.remove_if(|s| *s == "--debug"   || *s == "-d").is_some();
            let release   = args.remove_if(|s| *s == "--release" || *s == "-r").is_some();
            let toolchain = parse_toolchain(args.remove_if(|s| s.starts_with("--toolchain=") || s.starts_with("-t=")))?;
            let install   = args.remove_if(|s| *s == "--install").is_some();
            let echo      = args.remove_if(|s| *s == "--echo").is_some();
            let verbose   = args.remove_if(|s| *s == "--verbose" || *s == "-v").is_some();
            let profile = parse_profile(args.remove_if(|s| s.starts_with("--profile=")), debug, release)?;
            Ok(Action::Test{ switches: BuildSwitches{ profile, toolchain, install, echo, verbose, is_test: true }, args })
        }
        "clean" | "c" => {
            if args.is_empty() {
                Ok(Action::Clean)
            } else {
                Err(Error::ExtraArgs("clean".to_string(), args))
            }
        }
        "clangd" => {
            if args.is_empty() {
                Ok(Action::Clangd)
            } else {
                Err(Error::ExtraArgs("gen".to_string(), args))
            }
        }
        "help" => {
            if args.is_empty() {
                Ok(Action::Help{ action: None })
            } else {
                let action = args.remove(0);
                if matches!(action.as_str(), "new"|"init"|"clean"|"build"|"run"|"test"|"clangd"|"toolchains") && args.is_empty() {
                    Ok(Action::Help{ action: Some(action) })
                } else {
                    Err(Error::BadAction(action))
                }
            }
        }
        act => Err(Error::BadAction(act.to_string())),
    }
}


trait RemoveIf {
    type Item;

    fn remove_if<P: FnMut(&Self::Item) -> bool>(&mut self, p: P) -> Option<Self::Item>;
}

impl<T> RemoveIf for Vec<T> {
    type Item = T;

    fn remove_if<P: FnMut(&Self::Item) -> bool>(&mut self, p: P) -> Option<Self::Item> {
        self.iter().position(p).map(|i| self.remove(i))
    }
}


fn parse_toolchain(toolchain: Option<String>) -> Result<ToolChain, Error> {
    if let Some(tc) = toolchain {
        let tc = if let Some(tc) = tc.strip_prefix("-t=") {
            tc.to_ascii_lowercase()
        } else {
            tc.strip_prefix("--toolchain=").unwrap().to_ascii_lowercase()
        };
        if tc == "msvc" {
            if cfg!(windows) {
                Ok(ToolChain::Msvc)
            } else {
                Err(Error::MsvcUnavailable)
            }
        } else if tc == "gcc" {
            Ok(ToolChain::Gcc)
        } else if tc == "clang" {
            if cfg!(windows) {
                Ok(ToolChain::ClangMsvc)
            } else {
                Ok(ToolChain::ClangGnu)
            }
        } else if tc == "clang-gnu" {
            Ok(ToolChain::ClangGnu)
        } else if tc == "clang-msvc" {
            if cfg!(windows) {
                Ok(ToolChain::ClangMsvc)
            } else {
                Err(Error::MsvcUnavailable)
            }
        } else if tc == "zig" {
            Ok(ToolChain::Zig)
        } else {
            Err(Error::UnknownToolChain(tc.to_string()))
        }
    } else {
        Ok(ToolChain::default())
    }
}

fn parse_profile(profile: Option<String>, debug: bool, release: bool) -> Result<Profile, Error> {
    if debug && release { return Err(Error::ExtraArgs("build".to_string(), vec![ "--release".to_string() ])) }
    if let Some(prof) = profile {
        if debug   { return Err(Error::ExtraArgs("build".to_string(), vec![ "--debug".to_string() ])) }
        if release { return Err(Error::ExtraArgs("build".to_string(), vec![ "--release".to_string() ])) }

        let prof = prof.strip_prefix("--profile=").unwrap();
        if prof == "debug" {
            Ok(Profile::Debug)
        } else if prof == "release" {
            Ok(Profile::Release)
        } else {
            Ok(Profile::Custom(prof.to_string()))
        }
    } else if release {
        Ok(Profile::Release)
    } else {
        Ok(Profile::Debug)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_action_new_1() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), name.clone() ];
        let result = parse_args(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: false, is_c: false, clangd: false });
    }

    #[test]
    pub fn parse_action_new_2() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), name.clone(), "--lib".to_string() ];
        let result = parse_args(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: false, clangd: false });
    }

    #[test]
    pub fn parse_action_new_3() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), "--lib".to_string(), name.clone() ];
        let result = parse_args(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: false, clangd: false });
    }

    #[test]
    pub fn parse_action_new_4() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), "--lib".to_string(), name.clone(), "--c".to_string() ];
        let result = parse_args(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: true, clangd: false });
    }


    #[test]
    pub fn parse_action_build_1() {
        let result = parse_args(vec![ "build".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ switches: BuildSwitches{ profile: Profile::Debug, ..Default::default() } });
    }

    #[test]
    pub fn parse_action_build_2() {
        let result = parse_args(vec![ "build".to_string(), "-r".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ switches: BuildSwitches{ profile: Profile::Release, ..Default::default() } });
    }

    #[test]
    pub fn parse_action_build_3() {
        let result = parse_args(vec![ "build".to_string(), "-t=gcc".to_string(), "--release".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{
            switches: BuildSwitches{ profile: Profile::Release, toolchain: ToolChain::Gcc, ..Default::default() }
        });
    }

    #[test]
    pub fn parse_action_build_4() {
        let result = parse_args(vec![ "build".to_string(), "-t=clang-gnu".to_string(), "--release".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{
            switches: BuildSwitches{ profile: Profile::Release, toolchain: ToolChain::ClangGnu, ..Default::default() }
        });
    }


    #[test]
    pub fn parse_action_run_1() {
        let result = parse_args(vec![ "run".to_string(), "--".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ switches: BuildSwitches::default(), args: vec![] });
    }

    #[test]
    pub fn parse_action_run_2() {
        let result = parse_args(vec![ "run".to_string(), "-r".to_string(), "--".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ switches: BuildSwitches{ profile: Profile::Release, ..Default::default() }, args: vec![] });
    }

    #[test]
    pub fn parse_action_run_3() {
        let result = parse_args(vec![ "run".to_string(), "--".to_string(), "-r".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ switches: BuildSwitches::default(), args: vec![ "-r".to_string() ] });
    }


    #[test]
    pub fn parse_action_error_1() {
        let result = parse_args(vec![ "abc".to_string(), "--release".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_2() {
        let result = parse_args(vec![ "build".to_string(), "dummy".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_3() {
        let result = parse_args(vec![ "build".to_string(), "--release".to_string(), "dummy".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_4() {
        let result = parse_args(vec![ "build".to_string(), "-d".to_string(), "-r".to_string() ]);
        assert!(result.is_err());
    }
}

