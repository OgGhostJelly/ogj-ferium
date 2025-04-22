#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use colored::Colorize as _;
        eprintln!("{}{}", "Warning! ".bright_yellow(), format!($($arg)*).bright_yellow())
    }};
}
