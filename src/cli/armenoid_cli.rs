use std::io::{BufReader, Write};
use serbzip::codecs::armenoid::Armenoid;
use serbzip::codecs::Codec;
use crate::cli::app_error::AppError;
use crate::cli::{Args, banner, Mode};
use crate::cli::banner::{BLUE, RED, YELLOW};

pub(in super) fn run(args: Args) -> Result<(), AppError> {
    banner::print(r#"

 █████╗ ██████╗ ███╗   ███╗███████╗███╗   ██╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗████╗ ████║██╔════╝████╗  ██║██╔═══██╗██║██╔══██╗
███████║██████╔╝██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║██║██║  ██║
██╔══██║██╔══██╗██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║██║██║  ██║
██║  ██║██║  ██║██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝██║██████╔╝
╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═════╝

    "#, &[RED, RED, BLUE, BLUE, YELLOW, YELLOW]);

    let mode = args.mode()?;
    let input_reader = args.input_reader()?;
    let mut output_writer: Box<dyn Write> = args.output_writer()?;
    let codec = Armenoid::default();
    match mode {
        Mode::Compress => codec.compress(&mut BufReader::new(input_reader), &mut output_writer)?,
        Mode::Expand => codec.expand(&mut BufReader::new(input_reader), &mut output_writer)?,
    }
    output_writer.flush()?;
    Ok(())
}