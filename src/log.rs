
#[macro_export]
macro_rules! log_info {
    () => { {
        let mut termcolor_stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut termcolor_stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Green))).unwrap();
        writeln!(&mut termcolor_stdout, "[VanGo:  info]").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut termcolor_stdout).unwrap();
    } };
    ($($arg:tt)*) => { {
        let mut termcolor_stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut termcolor_stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Green))).unwrap();
        write!(&mut termcolor_stdout, "[VanGo:  info] ").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut termcolor_stdout).unwrap();
        writeln!(&mut termcolor_stdout, $($arg)*).unwrap();
    } };
}

#[macro_export]
macro_rules! log_warn {
    () => { {
        let mut termcolor_stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut termcolor_stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Yellow))).unwrap();
        writeln!(&mut termcolor_stdout, "[VanGo:  warn]").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut termcolor_stdout).unwrap();
    } };
    ($($arg:tt)*) => { {
        let mut termcolor_stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut termcolor_stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Yellow))).unwrap();
        write!(&mut termcolor_stdout, "[VanGo:  warn] ").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut termcolor_stdout).unwrap();
        writeln!(&mut termcolor_stdout, $($arg)*).unwrap();
    } };
}

#[macro_export]
macro_rules! log_error {
    () => { {
        let mut termcolor_stderr = ::termcolor::StandardStream::stderr(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut termcolor_stderr, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Red))).unwrap();
        writeln!(&mut termcolor_stderr, "[VanGo: error]").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut termcolor_stderr).unwrap();
    } };
    ($($arg:tt)*) => { {
        let mut termcolor_stderr = ::termcolor::StandardStream::stderr(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut termcolor_stderr, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Red))).unwrap();
        write!(&mut termcolor_stderr, "[VanGo: error] ").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut termcolor_stderr).unwrap();
        writeln!(&mut termcolor_stderr, $($arg)*).unwrap();
    } };
}

