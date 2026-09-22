#[macro_use]
mod log;

use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand, ValueEnum};
use std::str::FromStr;
use crate::config::{Profile, ToolChain};


const PROFILES_HELP: &str = "\
Profiles:
    debug    No optimization; Generate debugging information; 'VANGO_DEBUG' macro defined; Generally faster compile times
    release  High optimization; 'VANGO_RELEASE' macro defined; Generally slower compile times";


#[derive(Parser, Debug)]
#[command(
    name = "vango",
    help_template = concat!("VanGo ", env!("CARGO_PKG_VERSION"), " - C/C++ build automation tool\n\n", "{usage-heading} {usage}\n\n{all-args}"),
    subcommand_value_name = "ACTION",
    disable_help_flag = true,
    disable_help_subcommand = true,
    args_conflicts_with_subcommands = true
)]
struct Cli {
    /// Print basic help info
    #[arg(short, long, action = ArgAction::Help)]
    help: Option<bool>,
    /// Print version and exit
    #[arg(short, long)]
    version: bool,
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Action {
    #[command(
        about = "Create a new empty project",
        long_about = "Create a new directory with a boilerplate C++ project",
        override_usage = "vango new <NAME> [OPTIONS]"
    )]
    New {
        /// Name of the project directory
        name: String,
        #[command(flatten)]
        opts: ProjectOpts,
    },

    #[command(
        about = "Create an empty project in an existing location",
        long_about = "Create a boilerplate C++ project inside an existing directoy"
    )]
    Init {
        #[command(flatten)]
        opts: ProjectOpts,
    },

    /// Display help about a command (list toolchains with 'help toolchains')
    Help {
        #[arg(value_parser = ["new", "init", "clean", "gen", "build", "run", "test", "toolchains"])]
        action: Option<String>,
    },

    /// Remove all generated build files from the current project
    #[command(visible_alias = "c")]
    Clean,

    /// Build the current project
    #[command(visible_alias = "b", after_help = PROFILES_HELP)]
    Build {
        #[command(flatten)]
        build: BuildArgs,
    },

    #[command(
        visible_alias = "r",
        about = "Build the current project and run it",
        long_about = "Build and run the current project, forwarding command-line arguments, with project root as working directory",
        override_usage = "vango run [OPTIONS] [-- ARGS]",
        after_help = PROFILES_HELP
    )]
    Run {
        #[command(flatten)]
        build: BuildArgs,
        /// Arguments forwarded to the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    #[command(
        visible_alias = "t",
        about = "Build the current project and test it",
        long_about = "Build the current project and run all (or select) tests in 'test' directory. Defines 'VANGO_TEST'",
        override_usage = "vango test [OPTIONS] [TESTS]",
        after_help = PROFILES_HELP
    )]
    Test {
        #[command(flatten)]
        build: BuildArgs,
        /// Run only these tests
        #[arg(value_name = "TESTS")]
        args: Vec<String>,
    },

    #[command(
        about = "Generate 'compile_commands.json' for the current project",
        long_about = "Generate auxiliary files corresponding to the current project (clangd 'compile_commands.json')",
        override_usage = "vango gen clangd"
    )]
    Gen {
        #[arg(value_enum)]
        target: GenTarget,
    },

    /// Print version and exit
    #[command(hide = true)]
    Version,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenTarget {
    Clangd,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ProjectOpts {
    /// Generate library boilerplate instead of application
    #[arg(long = "lib")]
    pub library: bool,
    /// Include options 'warn-level="high"' and 'iso-compliant=true'
    #[arg(long)]
    pub strict: bool,
    /// Generate C boilerplate instead of C++
    #[arg(long = "c")]
    pub is_c: bool,
    /// Generate 'compile_flags.txt' based on default settings
    #[arg(long)]
    pub clangd: bool,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildArgs {
    /// Build project in debug profile (default)
    #[arg(short, long, conflicts_with_all = ["release", "profile"])]
    debug: bool,
    /// Build project in release profile
    #[arg(short, long, conflicts_with = "profile")]
    release: bool,
    /// Specify compilation profile (debug, release, or custom)
    #[arg(short, long, value_name = "PROF", require_equals = true)]
    profile: Option<String>,
    #[arg(short, long, value_name = "TOOL", require_equals = true, value_parser = parse_toolchain, help = toolchain_help())]
    toolchain: Option<ToolChain>,
    /// On unix-like systems: installs headers and binaries into /usr/local/* on build
    #[arg(long, hide = true)]
    install: bool,
    /// Echo the entire build command composed by vango
    #[arg(long)]
    echo: bool,
    /// Forward '--verbose' to invoked tool, if available
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildSwitches {
    pub profile: Profile,
    pub toolchain: Option<ToolChain>,
    pub install: bool,
    pub echo: bool,
    pub verbose: bool,
    pub is_test: bool,
}

impl BuildArgs {
    pub fn into_switches(self, is_test: bool) -> BuildSwitches {
        let profile = match self.profile {
            Some(p) if p == "debug" => Profile::Debug,
            Some(p) if p == "release" => Profile::Release,
            Some(p) => Profile::Custom(p),
            None if self.release => Profile::Release,
            None => Profile::Debug,
        };
        BuildSwitches {
            profile,
            toolchain: self.toolchain,
            install: self.install,
            echo: self.echo,
            verbose: self.verbose,
            is_test,
        }
    }
}

fn toolchain_help() -> String {
    // let default = ToolChain::user_default().map(|t| t.to_string()).unwrap_or_default();
    let default = ToolChain::default();
    format!("Specify a toolchain for compilation (user default: {default})")
}

fn parse_toolchain(s: &str) -> Result<ToolChain, String> {
    ToolChain::from_str(s).map_err(|e| e.to_string())
}

pub fn command() -> clap::Command {
    Cli::command()
}

pub fn collect_args() -> Action {
    let cli = Cli::parse();
    if cli.version {
        return Action::Version;
    }
    cli.action.unwrap_or(Action::Help { action: None })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn release_profile() {
        let cli = Cli::try_parse_from(["vango", "b", "-r"]).unwrap();
        let Some(Action::Build { build }) = cli.action else { panic!() };
        assert_eq!(build.into_switches(false).profile, Profile::Release);
    }

    #[test]
    fn debug_and_release_conflict() {
        assert!(Cli::try_parse_from(["vango", "build", "-d", "-r"]).is_err());
    }

    #[test]
    fn no_args_is_help() {
        let cli = Cli::try_parse_from(["vango"]).unwrap();
        assert_eq!(cli.action, None);
    }
}

