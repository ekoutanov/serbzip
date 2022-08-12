//! Utility for printing an ASCII banner, and using ANSI control codes to alter the foreground colour.

pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[371m";
pub const RESET: &str = "\x1b[0m";

/// Picks a control code from an 8-bit colour palette.
pub fn colour_256(code: u8) -> String {
    format!("\x1b[38;5;${code}m")
}

/// Prints the given logo stderr, decorating it with the supplied colour codes.
///
/// The logo is rendered line by line, simultaneously iterating over the `logo_lines` slice and
/// the `colours` slice.
///
/// If a line is empty (after all whitespace characters have trimmed), the line is
/// printed without decorations. Otherwise, if the line is nonempty, it is matched to the next
/// ANSI code in the given `colours` slice.
///
/// # Panics
/// If there is a nonempty line in the logo that does not correspond to a colour code.
/// In other words, the `colours` slice must be at least as long as the number of nonempty
/// lines in the `logo_lines` slice.
pub fn print(logo_lines: &str, colours: &[&str]) {
    let mut colours = colours.iter();
    for logo_line in logo_lines.split('\n') {
        if logo_line.trim().is_empty() {
            eprintln!();
        } else {
            let colour = colours.next().unwrap();
            eprintln!("{colour}{logo_line}{reset}", reset = RESET);
        }
    }
}