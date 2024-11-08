use crate::{
    repr::Config,
    error::Error,
};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdInput {
    pub action: Action,
    pub config: Config,
    pub args: Vec<String>,
}

pub fn parse_input(args: Vec<String>) -> Result<CmdInput, Error> {
    if args.len() == 1 { return Err(Error::MissingAction) }

    let action = match args[1].as_str() {
        "build"|"b" => Action::Build,
        "run"  |"r" => Action::Run,
        "clean"|"c" => Action::Clean,
        _ => return Err(Error::BadAction(args[1].clone())),
    };

    if action == Action::Clean && args.len() > 2 {
        return Err(Error::BadAction(args[2].clone()))
    }

    let (config, skip) = match args.get(2).map(|a| a.as_str()) {
        Some("-debug"  |"-d") => (Config::Debug, 3),
        Some("-release"|"-r") => (Config::Release, 3),
        _ => (Config::Debug, 2)
    };

    Ok(CmdInput{
        action,
        config,
        args: args.into_iter().skip(skip).collect(),
    })
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Build,
    Run,
    Clean,
}

impl Action {
    pub fn build(&self) -> bool { *self != Action::Clean }
    pub fn run(&self)   -> bool { *self == Action::Run }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_input_simple() {
        let expected = CmdInput{ action: Action::Build, config: Config::Debug, args: vec![] };

        let args = vec![ "mscmp".to_string(), "build".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "b".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "build".to_string(), "-debug".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "b".to_string(), "-debug".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "build".to_string(), "-d".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "b".to_string(), "-d".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);
    }

    #[test]
    pub fn test_get_input_with_args() {
        let expected = CmdInput{ action: Action::Run, config: Config::Debug, args: vec![ "abc".to_string(), "def".to_string() ]  };

        let args = vec![ "mscmp".to_string(), "run".to_string(), "abc".to_string(), "def".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "r".to_string(), "abc".to_string(), "def".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let expected = CmdInput{ action: Action::Run, config: Config::Release, args: vec![ "abc".to_string(), "def".to_string() ] };

        let args = vec![ "mscmp".to_string(), "run".to_string(), "-release".to_string(), "abc".to_string(), "def".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "r".to_string(), "-release".to_string(), "abc".to_string(), "def".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "run".to_string(), "-release".to_string(), "abc".to_string(), "def".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);

        let args = vec![ "mscmp".to_string(), "r".to_string(), "-release".to_string(), "abc".to_string(), "def".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);
    }

    #[test]
    pub fn test_get_input_failures() {
        let args = vec![ "mscmp".to_string(), "abc".to_string() ];
        assert!(parse_input(args).is_err());

        let args = vec![ "mscmp".to_string(), "abc".to_string(), "run".to_string() ];
        assert!(parse_input(args).is_err());

        let args = vec![ "mscmp".to_string(), "abc".to_string(), "r".to_string() ];
        assert!(parse_input(args).is_err());

        let expected = CmdInput{ action: Action::Run, config: Config::Debug, args: vec![ "abc".to_string(), "-release".to_string() ]  };

        let args = vec![ "mscmp".to_string(), "run".to_string(), "abc".to_string(), "-release".to_string() ];
        assert_eq!(parse_input(args).unwrap(), expected);
    }
}

