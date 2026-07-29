use colored::{ColoredString, Colorize};

const VESPER_ORANGE: (u8, u8, u8) = (255, 199, 153);
const VESPER_PEPPERMINT: (u8, u8, u8) = (153, 255, 228);
const VESPER_RED: (u8, u8, u8) = (255, 128, 128);

pub fn accent(value: impl ToString) -> ColoredString {
    let (red, green, blue) = VESPER_ORANGE;
    value.to_string().truecolor(red, green, blue).bold()
}

pub fn success(value: impl ToString) -> ColoredString {
    let (red, green, blue) = VESPER_PEPPERMINT;
    value.to_string().truecolor(red, green, blue)
}

pub fn danger(value: impl ToString) -> ColoredString {
    let (red, green, blue) = VESPER_RED;
    value.to_string().truecolor(red, green, blue)
}

pub fn strong(value: impl ToString) -> ColoredString {
    value.to_string().bold()
}

pub fn muted(value: impl ToString) -> ColoredString {
    value.to_string().dimmed()
}

pub fn warning(message: impl std::fmt::Display) {
    eprintln!("{} {message}", accent("warning:"));
}

pub fn error(message: impl std::fmt::Display) {
    eprintln!("{} {message}", danger("error:").bold());
}
