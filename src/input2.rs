use crate::{error::Error, config::{Profile, ToolChain}};
use clap::{Parser, Subcommand, Args};


#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Action {
    New {
        name: String,
        #[arg(default_value_t)]
        library: bool,
        #[arg(default_value_t)]
        is_c: bool,
        #[arg(default_value_t)]
        clangd: bool,
    },
    Init {
        #[arg(default_value_t)]
        library: bool,
        #[arg(default_value_t)]
        is_c: bool,
        #[arg(default_value_t)]
        clangd: bool,
    },
    Clean,
    Gen  { target: String },
    Build{ switches: BuildSwitches },
    Run  { switches: BuildSwitches, args: Vec<String> },
    Test { switches: BuildSwitches, args: Vec<String> },
    Help { action: Option<String> },
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Args)]
pub struct BuildSwitches {
    #[arg(value_enum, default_value_t=Profile::Debug, value_parser=valid_profile)]
    pub profile: Profile,

    #[arg(short, long, default_value_t=ToolChain::system_default(), value_name="TOOL", value_parser=valid_toolchain)]
    pub toolchain: ToolChain,

    #[arg(long)]
    pub crtstatic: bool,
    #[arg(long)]
    pub install: bool,
    #[arg(long)]
    pub echo: bool,
    #[arg(short, long)]
    pub verbose: bool,
}


fn valid_profile(pr: &str) -> Result<Profile, Error> {
}

fn valid_toolchain(tc: &str) -> Result<ToolChain, Error> {
    let tc = tc.to_ascii_lowercase();
    if cfg!(windows) {
        if tc == "msvc" {
            Ok(ToolChain::Msvc)
        } else if tc == "clang" || tc == "clang-msvc" {
            Ok(ToolChain::ClangMsvc)
        } else if tc == "clang-gnu" {
            Ok(ToolChain::ClangGnu)
        } else if tc == "gcc" {
            Ok(ToolChain::Gcc)
        } else if tc == "zig" {
            Ok(ToolChain::Zig)
        } else {
            Err(Error::UnknownToolChain(tc.to_string()))
        }
    } else {
        if tc == "msvc" || tc == "clang-msvc" {
            Err(Error::MsvcUnavailable)
        } else if tc == "clang" || tc == "clang-gnu" {
            Ok(ToolChain::ClangGnu)
        } else if tc == "gcc" {
            Ok(ToolChain::Gcc)
        } else if tc == "zig" {
            Ok(ToolChain::Zig)
        } else {
            Err(Error::UnknownToolChain(tc.to_string()))
        }
    }
}

