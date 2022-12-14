//! Entry point for the Armenoid codec.

use crate::cli::app_error::{AppError, CliError, CliErrorKind};
use crate::cli::banner::{BLUE, RED, YELLOW};
use crate::cli::{banner, compress_helper, expand_helper, Args, Mode};
use serbzip_core::codecs::armenoid::Armenoid;
use serbzip_core::succinct::CowStr;

pub(super) fn run(args: &Args) -> Result<(), AppError> {
    if !args.quiet() {
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
    };

    match args.mode()? {
        Mode::Compress => compress_helper(args, &Armenoid::default()),
        Mode::Expand => expand_helper(args, &Armenoid::default()),
        Mode::Compile => Err(AppError::from(CliError(
            CliErrorKind::UnsupportedMode,
            CowStr::Borrowed("unsupported mode for this codec"),
        ))),
    }
}
