use std::io::BufRead;
use crate::config::ToolChain;


pub fn help(action: Option<&String>) {
    let print_build_details = || {
        println!("Options:");
        println!("  -d, --debug             Build project in debug profile (default)");
        println!("  -r, --release           Build project in release profile");
        println!("  -p, --profile=<PROF>    Specify compilation profile (debug, release, or custom)");
        println!("  -t, --toolchain=<TOOL>  Specify a toolchain for compilation (user default: {})", ToolChain::default());
        // println!("      --install           On unix-like systems: installs headers and binaries into /usr/local/* on build");
        // println!("      --crtstatic         Link statically with the C runtime library");
        println!("      --echo              Echo the entire build command composed by vango");
        println!("  -v, --verbose           Forward '--verbose' to invoked tool, if available");
        println!();
        println!("Profiles:");
        println!("    debug    No optimization; Generate debugging information; 'VANGO_DEBUG' macro defined; Generally faster compile times");
        println!("    release  High optimization; 'VANGO_RELEASE' macro defined; Generally slower compile times");
    };

    if let Some(action) = action {
        match action.as_str() {
            "new" => {
                println!("Create a new folder with a boilerplate C++ project");
                println!();
                println!("Usage: vango new <NAME> [OPTIONS]");
                println!();
                println!("Options:");
                println!("    --lib     Generate library boilerplate instead of application");
                println!("    --c       Generate C boilerplate instead of C++");
                println!("    --clangd  Generate 'compile_flags.txt' based on default settings");
            }
            "init" => {
                println!("Create a boilerplate C++ project inside an existing folder");
                println!();
                println!("Usage: vango init [OPTIONS]");
                println!();
                println!("Options:");
                println!("    --lib     Generate library boilerplate instead of application");
                println!("    --c       Generate C boilerplate instead of C++");
                println!("    --clangd  Generate 'compile_flags.txt' based on default settings");
            }
            "clean" => {
                println!("Remove all generated build files from the current project");
                println!();
                println!("Usage: vango clean");
            }
            "clangd" => {
                println!("Generate 'compile_flags.txt' corresponding to the current project (language standard, include dirs, definitions");
                println!();
                println!("Usage: vango clangd");
            }
            "build" => {
                println!("Build the current project");
                println!();
                println!("Usage: vango build [OPTIONS]");
                println!();
                print_build_details();
            }
            "run" => {
                println!("Build and run the current project, forwarding command-line arguments, with project root as working directory");
                println!();
                println!("Usage: vango run [OPTIONS] [-- ARGS]");
                println!();
                print_build_details();
            }
            "test" => {
                println!("Build the current project and run all (or select) tests in 'test' directory. Defines 'VANGO_TEST'.");
                println!();
                println!("Usage: vango test [OPTIONS] [TESTS]");
                println!();
                print_build_details();
            }
            "toolchains" => {
                println!("Toolchains currently installed on this system:");
                println!();
                if let Ok(ver) = std::process::Command::new("gcc").arg("--version").output() {
                    println!("    gcc    - {}", ver.stdout.lines().next().unwrap().unwrap());
                } else {
                    println!("    gcc    - unavailable");
                }
                if let Ok(ver) = std::process::Command::new("clang").arg("--version").output() {
                    println!("    clang  - {}", ver.stdout.lines().next().unwrap().unwrap());
                } else {
                    println!("    clang  - unavailable");
                }
                if let Ok(ver) = std::process::Command::new("cl.exe").arg("--version").output() {
                    println!("    msvc   - {}", ver.stderr.lines().next().unwrap().unwrap());
                } else {
                    println!("    msvc   - unavailable");
                }
                if let Ok(ver) = std::process::Command::new("zig").arg("version").output() {
                    println!("    zig    - {}", ver.stdout.lines().next().unwrap().unwrap());
                } else {
                    println!("    zig    - unavailable");
                }
            }
            _ => (),
        }
    } else {
        println!("VanGo {} - C/C++ build automation tool", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Usage: vango [ACTION] [OPTIONS]");
        println!();
        println!("Commands:");
        println!("    new         Create a new empty project");
        println!("    init        Create an empty project in an existing location");
        println!("    help        Display help about a command (list toolchains with 'help toolchains')");
        println!("    clean, c    Remove all generated build files from the current project");
        println!("    build, b    Build the current project");
        println!("    run,   r    Build the current project and run it");
        println!("    test,  t    Build the current project and test it");
        println!("    clangd      Generate 'compile_flags.txt' for the current project");
    }
    println!();
}

