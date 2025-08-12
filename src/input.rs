use crate::{error::Error, repr::{Config, ToolChain}};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    New { library: bool, is_c: bool, name: String },
    Init{ library: bool, is_c: bool },
    Clean,
    Build{ switches: BuildSwitches },
    Run  { switches: BuildSwitches, args: Vec<String> },
    Test { switches: BuildSwitches, args: Vec<String> },
    Help { action: Option<String> },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BuildSwitches {
    pub config: Config,
    pub toolchain: ToolChain,
    pub crtstatic: bool,
    pub verbose: bool
}



pub fn parse_input(mut args: Vec<String>) -> Result<Action, Error> {
    if args.is_empty() {
        return Ok(Action::Help{ action: None })
    }

    match args.remove(0).as_str() {
        "new" => {
            let library = args.remove_if(|s| *s == "--lib").is_some();
            let is_c = args.remove_if(|s| *s == "--c").is_some();
            if args.len() == 1 {
                Ok(Action::New{ library, is_c, name: args.remove(0) })
            } else {
                Err(Error::ExtraArgs("new".to_string(), args))
            }
        }
        "init" => {
            let library = args.remove_if(|s| *s == "--lib").is_some();
            let is_c = args.remove_if(|s| *s == "--c").is_some();
            if args.is_empty() {
                Ok(Action::Init{ library, is_c })
            } else {
                Err(Error::ExtraArgs("init".to_string(), args))
            }
        }
        "build" | "b" => {
            let debug = args.remove_if(|s| *s == "-d" || *s == "--debug").is_some();
            let release = args.remove_if(|s| *s == "-r" || *s == "--release").is_some();
            let crtstatic = args.remove_if(|s| *s == "--crtstatic").is_some();
            let verbose = args.remove_if(|s| *s == "-v" || *s == "--verbose").is_some();
            if debug && release { return Err(Error::ExtraArgs("build".to_string(), vec![ "--release".to_string() ])) }
            let toolchain = parse_toolchain(args.remove_if(|s| s.starts_with("-t=") || s.starts_with("--toolchain=")))?;
            let config = if release { Config::Release } else { Config::Debug };
            if args.is_empty() {
                Ok(Action::Build{ switches: BuildSwitches{ config, toolchain, crtstatic, verbose } })
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
            let debug = args.remove_if(|s| *s == "-d" || *s == "--debug").is_some();
            let release = args.remove_if(|s| *s == "-r" || *s == "--release").is_some();
            let crtstatic = args.remove_if(|s| *s == "--crtstatic").is_some();
            let verbose = args.remove_if(|s| *s == "-v" || *s == "--verbose").is_some();
            if debug && release { return Err(Error::ExtraArgs("run".to_string(), vec![ "--release".to_string() ])) }
            let toolchain = parse_toolchain(args.remove_if(|s| s.starts_with("-t=") || s.starts_with("--toolchain=")))?;
            let config = if release { Config::Release } else { Config::Debug };
            if args.is_empty() {
                Ok(Action::Run{ switches: BuildSwitches{ config, toolchain, crtstatic, verbose }, args: user_args })
            } else {
                Err(Error::ExtraArgs("run".to_string(), args))
            }
        }
        "test" | "t" => {
            let debug = args.remove_if(|s| *s == "-d" || *s == "--debug").is_some();
            let release = args.remove_if(|s| *s == "-r" || *s == "--release").is_some();
            let crtstatic = args.remove_if(|s| *s == "--crtstatic").is_some();
            let verbose = args.remove_if(|s| *s == "-v" || *s == "--verbose").is_some();
            if debug && release { return Err(Error::ExtraArgs("test".to_string(), vec![ "--release".to_string() ])) }
            let toolchain = parse_toolchain(args.remove_if(|s| s.starts_with("-t=") || s.starts_with("--toolchain=")))?;
            let config = if release { Config::Release } else { Config::Debug };
            Ok(Action::Test{ switches: BuildSwitches{ config, toolchain, crtstatic, verbose }, args })
        }
        "clean" | "c" => {
            if args.is_empty() {
                Ok(Action::Clean)
            } else {
                Err(Error::ExtraArgs("clean".to_string(), args))
            }
        }
        "help" => {
            if args.is_empty() {
                Ok(Action::Help{ action: None })
            } else {
                let action = args.remove(0);
                if matches!(action.as_str(), "new"|"init"|"clean"|"build"|"run"|"test") && args.is_empty() {
                    Ok(Action::Help{ action: Some(action) })
                } else {
                    Err(Error::ExtraArgs("help".to_string(), args))
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
        let tc = tc.strip_prefix("-t=").unwrap();
        if tc == "msvc" {
            if cfg!(target_os = "windows") {
                Ok(ToolChain::Msvc)
            } else {
                Err(Error::MsvcUnavailable)
            }
        } else if tc == "gnu" {
            Ok(ToolChain::Gnu)
        } else if tc == "clang" {
            Ok(ToolChain::Clang)
        } else if tc == "zig" {
            Ok(ToolChain::Zig)
        } else {
            Err(Error::UnknownToolChain(tc.to_string()))
        }
    } else {
        Ok(ToolChain::default())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_action_new_1() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), name.clone() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: false, is_c: false });
    }

    #[test]
    pub fn parse_action_new_2() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), name.clone(), "--lib".to_string() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: false });
    }

    #[test]
    pub fn parse_action_new_3() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), "--lib".to_string(), name.clone() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: false });
    }

    #[test]
    pub fn parse_action_new_4() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), "--lib".to_string(), name.clone(), "--c".to_string() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: true });
    }


    #[test]
    pub fn parse_action_build_1() {
        let result = parse_input(vec![ "build".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ switches: BuildSwitches{ config: Config::Debug, ..Default::default() } });
    }

    #[test]
    pub fn parse_action_build_2() {
        let result = parse_input(vec![ "build".to_string(), "-r".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ switches: BuildSwitches{ config: Config::Release, ..Default::default() } });
    }

    #[test]
    pub fn parse_action_build_3() {
        let result = parse_input(vec![ "build".to_string(), "-t=gnu".to_string(), "--release".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{
            switches: BuildSwitches{ config: Config::Release, toolchain: ToolChain::Gnu, ..Default::default() }
        });
    }

    #[test]
    pub fn parse_action_build_4() {
        let result = parse_input(vec![ "build".to_string(), "-t=clang".to_string(), "--release".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{
            switches: BuildSwitches{ config: Config::Release, toolchain: ToolChain::Clang, ..Default::default() }
        });
    }


    #[test]
    pub fn parse_action_run_1() {
        let result = parse_input(vec![ "run".to_string(), "--".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ switches: BuildSwitches::default(), args: vec![] });
    }

    #[test]
    pub fn parse_action_run_2() {
        let result = parse_input(vec![ "run".to_string(), "-r".to_string(), "--".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ switches: BuildSwitches{ config: Config::Release, ..Default::default() }, args: vec![] });
    }

    #[test]
    pub fn parse_action_run_3() {
        let result = parse_input(vec![ "run".to_string(), "--".to_string(), "-r".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ switches: BuildSwitches::default(), args: vec![ "-r".to_string() ] });
    }


    #[test]
    pub fn parse_action_error_1() {
        let result = parse_input(vec![ "abc".to_string(), "--release".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_2() {
        let result = parse_input(vec![ "build".to_string(), "dummy".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_3() {
        let result = parse_input(vec![ "build".to_string(), "--release".to_string(), "dummy".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_4() {
        let result = parse_input(vec![ "build".to_string(), "-d".to_string(), "-r".to_string() ]);
        assert!(result.is_err());
    }
}

