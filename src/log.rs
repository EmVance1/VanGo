

#[macro_export]
macro_rules! log_info {
    () => { {
        let mut stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Green))).unwrap();
        writeln!(&mut stdout, "[mscmp:  info]").unwrap();
    } };
    ($($arg:tt)*) => { {
        let mut stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Green))).unwrap();
        write!(&mut stdout, "[mscmp:  info] ").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut stdout).unwrap();
        writeln!(&mut stdout, $($arg)*).unwrap();
    } };
}

#[macro_export]
macro_rules! log_info_noline {
    () => { {
        let mut stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Green))).unwrap();
        write!(&mut stdout, "[mscmp:  info]").unwrap();
    } };
    ($($arg:tt)*) => { {
        let mut stdout = ::termcolor::StandardStream::stdout(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut stdout, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Green))).unwrap();
        write!(&mut stdout, "[mscmp:  info] ").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut stdout).unwrap();
        write!(&mut stdout, $($arg)*).unwrap();
    } };
}


#[macro_export]
macro_rules! log_error {
    () => { {
        let mut stderr = ::termcolor::StandardStream::stderr(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut stderr, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Red))).unwrap();
        writeln!(&mut stderr, "[mscmp: error]").unwrap();
    } };
    ($($arg:tt)*) => { {
        let mut stderr = ::termcolor::StandardStream::stderr(::termcolor::ColorChoice::Always);
        <::termcolor::StandardStream as ::termcolor::WriteColor>::set_color(&mut stderr, ::termcolor::ColorSpec::new()
            .set_fg(Some(::termcolor::Color::Red))).unwrap();
        write!(&mut stderr, "[mscmp: error] ").unwrap();
        <::termcolor::StandardStream as ::termcolor::WriteColor>::reset(&mut stderr).unwrap();
        writeln!(&mut stderr, $($arg)*).unwrap();
    } };
}

