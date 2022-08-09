pub mod cli;

use std::process;

fn main() {
    cli::run().unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        process::exit(1)
    });
}