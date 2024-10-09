use std::fmt::Display;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdInput {
    pub action: Action,
    pub config: Config,
    pub args: Vec<String>,
}

pub fn get_input() -> Result<CmdInput, String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 { return Err("no action provided [build, run, clean]".to_string()) }

    let action = match args[1].as_str() {
        "build"|"b" => Action::Build,
        "run"  |"r" => Action::Run,
        "clean"|"c" => Action::Clean,
        _ => return Err("invalid action provided [build, run, clean]".to_string()),
    };

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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    Debug,
    Release,
}

impl Config {
    // pub fn is_debug  (&self) -> bool { *self == Config::Debug }
    pub fn is_release(&self) -> bool { *self == Config::Release }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug   => write!(f, "DEBUG"),
            Self::Release => write!(f, "RELEASE"),
        }
    }
}

