use crate::cli;

pub fn version() {
    println!("VanGo {} - C/C++ build automation tool", env!("CARGO_PKG_VERSION"));
}

pub fn help(action: Option<&str>) {
    let mut cmd = cli::command();
    cmd.build();
    let result = match action {
        Some("toolchains") => {
            toolchains();
            return;
        }
        Some(name) => cmd.find_subcommand_mut(name).expect("unknown action").print_long_help(),
        None => cmd.print_help(),
    };
    let _ = result;
}

fn toolchains() {
    println!("Toolchains currently installed on this system:");
    println!();
    let probes = [
        ("gcc", "gcc", "--version", false),
        ("clang", "clang", "--version", false),
        ("msvc", "cl.exe", "--version", true),
        ("zig", "zig", "version", false),
        ("emcc", "emcc.bat", "--version", false),
    ];
    for (label, exe, arg, use_stderr) in probes {
        let line = std::process::Command::new(exe).arg(arg).output().ok().and_then(|out| {
            let bytes = if use_stderr { out.stderr } else { out.stdout };
            String::from_utf8_lossy(&bytes).lines().next().map(str::to_owned)
        });
        println!("    {label:<6} - {}", line.as_deref().unwrap_or("unavailable"));
    }
    println!();
}
