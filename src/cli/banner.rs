pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[371m";
pub const RESET: &str = "\x1b[0m";

pub fn colour_256(code: u8) -> String {
    format!("\x1b[38;5;${code}m")
}

pub fn print(logo: &str, colours: &[&str]) {
    let mut colours = colours.into_iter();
    for line in logo.split("\n") {
        if !line.trim().is_empty() {
            let foreground = colours.next().unwrap();
            eprintln!("{foreground}{line}{reset}", reset = RESET);
        }
    }
}