pub mod gnu;
pub mod msvc;

pub fn echo_command(cmd: &std::process::Command) {
    print!("{} ", cmd.get_program().display());
    for arg in cmd.get_args() {
        print!("{} ", arg.display());
    }
    println!();
}
