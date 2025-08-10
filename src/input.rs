use crate::{error::Error, repr::Config};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    New {
        name: String,
        library: bool,
        is_c: bool,
    },
    Clean,
    Build {
        config: Config,
        mingw: bool,
    },
    Run {
        config: Config,
        mingw: bool,
        args: Vec<String>,
    },
    Test {
        config: Config,
        mingw: bool,
        args: Vec<String>,
    },
}


pub fn parse_input(mut args: Vec<String>) -> Result<Action, Error> {
    if args.is_empty() {
        return Err(Error::MissingAction);
    }

    match args.remove(0).as_str() {
        "new" | "n" => {
            let library = args.remove_if(|s| *s == "-lib").is_some();
            let is_c = args.remove_if(|s| *s == "-c").is_some();
            if args.len() == 1 {
                let name = args.remove(0);
                Ok(Action::New{ name, library, is_c })
            } else {
                Err(Error::ExtraArgs("new".to_string(), args))
            }
        }
        "clean" | "c" => {
            if args.is_empty() {
                Ok(Action::Clean)
            } else {
                Err(Error::ExtraArgs("clean".to_string(), args))
            }
        }
        "build" | "b" => {
            let debug = args.remove_if(|s| *s == "-d" || *s == "-debug").is_some();
            let release = args.remove_if(|s| *s == "-r" || *s == "-release").is_some();
            if debug && release { return Err(Error::ExtraArgs("test".to_string(), vec![ "-release".to_string() ])) }
            let mingw = args.remove_if(|s| *s == "-mingw").is_some();
            let config = if release { Config::Release } else { Config::Debug };
            if args.is_empty() {
                Ok(Action::Build{ config, mingw })
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
            let debug = args.remove_if(|s| *s == "-d" || *s == "-debug").is_some();
            let release = args.remove_if(|s| *s == "-r" || *s == "-release").is_some();
            if debug && release { return Err(Error::ExtraArgs("test".to_string(), vec![ "-release".to_string() ])) }
            let mingw = args.remove_if(|s| *s == "-mingw").is_some();
            let config = if release { Config::Release } else { Config::Debug };
            Ok(Action::Run{ config, mingw, args: user_args })
        }
        "test" | "t" => {
            let debug = args.remove_if(|s| *s == "-d" || *s == "-debug").is_some();
            let release = args.remove_if(|s| *s == "-r" || *s == "-release").is_some();
            if debug && release { return Err(Error::ExtraArgs("test".to_string(), vec![ "-release".to_string() ])) }
            let mingw = args.remove_if(|s| *s == "-mingw").is_some();
            let config = if release { Config::Release } else { Config::Debug };
            Ok(Action::Test{ config, mingw, args })
        }
        _ => Err(Error::BadAction(args[1].clone())),
    }
}


trait RemoveIf {
    type Item;

    fn remove_if<P: FnMut(&Self::Item) -> bool>(&mut self, p: P) -> Option<Self::Item>;
}

impl<T> RemoveIf for Vec<T> {
    type Item = T;

    fn remove_if<P: FnMut(&Self::Item) -> bool>(&mut self, p: P) -> Option<Self::Item> {
        self.iter().position(p).and_then(|i| Some(self.remove(i)))
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
        let action = vec![ "new".to_string(), name.clone(), "-lib".to_string() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: false });
    }

    #[test]
    pub fn parse_action_new_3() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), "-lib".to_string(), name.clone() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: false });
    }

    #[test]
    pub fn parse_action_new_4() {
        let name = "foo".to_string();
        let action = vec![ "new".to_string(), "-lib".to_string(), name.clone(), "-c".to_string() ];
        let result = parse_input(action);
        assert_eq!(result.unwrap(), Action::New{ name, library: true, is_c: true });
    }


    #[test]
    pub fn parse_action_build_1() {
        let result = parse_input(vec![ "build".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ config: Config::Debug, mingw: false });
    }

    #[test]
    pub fn parse_action_build_2() {
        let result = parse_input(vec![ "build".to_string(), "-r".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ config: Config::Release, mingw: false });
    }

    #[test]
    pub fn parse_action_build_3() {
        let result = parse_input(vec![ "build".to_string(), "-mingw".to_string(), "-release".to_string() ]);
        assert_eq!(result.unwrap(), Action::Build{ config: Config::Release, mingw: true });
    }


    #[test]
    pub fn parse_action_run_1() {
        let result = parse_input(vec![ "run".to_string(), "--".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ config: Config::Debug, mingw: false, args: vec![] });
    }

    #[test]
    pub fn parse_action_run_2() {
        let result = parse_input(vec![ "run".to_string(), "-r".to_string(), "--".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ config: Config::Release, mingw: false, args: vec![] });
    }

    #[test]
    pub fn parse_action_run_3() {
        let result = parse_input(vec![ "run".to_string(), "--".to_string(), "-r".to_string() ]);
        assert_eq!(result.unwrap(), Action::Run{ config: Config::Debug, mingw: false, args: vec![ "-r".to_string() ] });
    }


    #[test]
    pub fn parse_action_error_1() {
        let result = parse_input(vec![ "build".to_string(), "-dummy".to_string() ]);
        assert!(result.is_err());
    }

    #[test]
    pub fn parse_action_error_2() {
        let result = parse_input(vec![ "build".to_string(), "-release".to_string(), "-dummy".to_string() ]);
        assert!(result.is_err());
    }
}

