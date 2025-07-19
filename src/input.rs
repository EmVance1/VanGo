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
            let config = match args.get(1).map(|a| a.as_str()) {
                Some("-debug"  |"-d") => Config::Debug,
                Some("-release"|"-r") => Config::Release,
                _ => Config::Debug
            };
            Ok(Action::Build{ config, mingw: false })
        }
        "run"|"r" => {
            let (config, count) = match args.get(1).map(|a| a.as_str()) {
                Some("-debug"  |"-d") => (Config::Debug, 1),
                Some("-release"|"-r") => (Config::Release, 1),
                _ => (Config::Debug, 0)
            };
            args.remove(0);
            if count == 1 {
                args.remove(0);
            }
            Ok(Action::Run{ config, mingw: false, args })
        }
        "test"|"t" => {
            let (config, count) = match args.get(1).map(|a| a.as_str()) {
                Some("-debug"  |"-d") => (Config::Debug, 1),
                Some("-release"|"-r") => (Config::Release, 1),
                _ => (Config::Debug, 0)
            };
            args.remove(0);
            if count == 1 {
                args.remove(0);
            }
            Ok(Action::Test{ config, mingw: false, args })
        }
        _ => Err(Error::BadAction(args[1].clone())),
    }
}

