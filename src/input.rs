use crate::{
    repr::Config,
    error::Error,
};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    New{ name: String, library: bool, isc: bool },
    #[allow(unused)]
    Set{ key: String, val: String },
    Clean,
    Build{ config: Config, mingw: bool },
    Run  { config: Config, mingw: bool, args: Vec<String> },
    Test { config: Config, mingw: bool, args: Vec<String> },
}

pub fn parse_input(mut args: Vec<String>) -> Result<Action, Error> {
    args.remove(0);
    if args.is_empty() { return Err(Error::MissingAction) }

    match args[0].as_str() {
        "new"|"n" => {
            if args.len() == 2 {
                Ok(Action::New{ name: args[1].clone(), library: false, isc: false })
            } else {
                let mut library = false;
                let mut isc = false;
                let mut safety = 0;
                while args.len() > 2 {
                    if let Some(pos) = args.iter().position(|s| *s == "-lib") {
                        args.remove(pos);
                        library = true;
                    } else if let Some(pos) = args.iter().position(|s| *s == "-c") {
                        args.remove(pos);
                        isc = true;
                    } else if safety > 2 {
                        return Err(Error::BadAction(args[1].clone()));
                    }
                    safety += 1;
                }
                Ok(Action::New{ name: args[1].clone(), library, isc })
            }
        }
        "clean"|"c" => {
            if args.len() == 1 {
                Ok(Action::Clean)
            } else {
                Err(Error::BadAction(args[2].clone()))
            }
        }
        "build"|"b" => {
            let mut config = Config::Debug;
            let mut mingw = false;
            args.remove(0);
            if let Some(pos) = args.iter().position(|s| *s == "-d" || *s == "-debug") {
                args.remove(pos);
                config = Config::Debug;
            }
            if let Some(pos) = args.iter().position(|s| *s == "-r" || *s == "-release") {
                args.remove(pos);
                config = Config::Release;
            }
            if let Some(pos) = args.iter().position(|s| *s == "-mingw") {
                args.remove(pos);
                mingw = true;
            }
            if !args.is_empty() { return Err(Error::BadAction(args[0].clone()))}
            Ok(Action::Build{ config, mingw })
        }
        "run"|"r" => {
            let mut config = Config::Debug;
            let mut mingw = false;
            args.remove(0);
            if let Some(pos) = args.iter().position(|s| *s == "-d" || *s == "-debug") {
                args.remove(pos);
                config = Config::Debug;
            }
            if let Some(pos) = args.iter().position(|s| *s == "-r" || *s == "-release") {
                args.remove(pos);
                config = Config::Release;
            }
            if let Some(pos) = args.iter().position(|s| *s == "-mingw") {
                args.remove(pos);
                mingw = true;
            }
            Ok(Action::Run{ config, mingw, args })
        }
        "test"|"t" => {
            let mut config = Config::Debug;
            let mut mingw = false;
            args.remove(0);
            if let Some(pos) = args.iter().position(|s| *s == "-d" || *s == "-debug") {
                args.remove(pos);
                config = Config::Debug;
            }
            if let Some(pos) = args.iter().position(|s| *s == "-r" || *s == "-release") {
                args.remove(pos);
                config = Config::Release;
            }
            if let Some(pos) = args.iter().position(|s| *s == "-mingw") {
                args.remove(pos);
                mingw = true;
            }
            Ok(Action::Test{ config, mingw, args })
        }
        _ => Err(Error::BadAction(args[1].clone())),
    }
}

