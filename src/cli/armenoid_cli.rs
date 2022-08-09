use crate::cli::app_error::{AppError, CliError, CliErrorKind};
use crate::cli::banner::{BLUE, RED, YELLOW};
use crate::cli::{banner, Args, Mode, compress_helper, expand_helper};
use serbzip::codecs::armenoid::Armenoid;
use serbzip::succinct::CowStr;

pub(super) fn run(args: &Args) -> Result<(), AppError> {
    banner::print(
        r#"

 █████╗ ██████╗ ███╗   ███╗███████╗███╗   ██╗ ██████╗ ██╗██████╗
██╔══██╗██╔══██╗████╗ ████║██╔════╝████╗  ██║██╔═══██╗██║██╔══██╗
███████║██████╔╝██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║██║██║  ██║
██╔══██║██╔══██╗██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║██║██║  ██║
██║  ██║██║  ██║██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝██║██████╔╝
╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═════╝

    "#,
        &[RED, RED, BLUE, BLUE, YELLOW, YELLOW],
    );

    match args.mode()? {
        Mode::Compress => compress_helper(args, &Armenoid::default()),
        Mode::Expand => expand_helper(args, &Armenoid::default()),
        Mode::Compile => Err(AppError::from(CliError(
            CliErrorKind::UnsupportedMode,
            CowStr::Borrowed("unsupported mode for this codec"),
        ))),
    }
}