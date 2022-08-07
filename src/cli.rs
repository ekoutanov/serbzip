use clap::Parser;
use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// A quasi-lossless Balkanoidal meta-lingual compressor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Compress
    #[clap(short, long, value_parser)]
    compress: bool,

    /// Expand
    #[clap(short = 'x', long, value_parser)]
    expand: bool,

    /// Dictionary file (.txt for a text file, .img for binary image)
    #[clap(short, long, value_parser)]
    dictionary: Option<String>,

    /// Input file (defaults to stdin)
    #[clap(short, long, value_parser)]
    input_file: Option<String>,

    /// Output file (defaults to stdout)
    #[clap(short, long, value_parser)]
    output_file: Option<String>,

    /// Output file for compiling the dictionary image
    #[clap(short = 'm', long, value_parser)]
    dictionary_image_output_file: Option<String>,
}

#[derive(Debug)]
pub enum Mode {
    Compress,
    Expand,
}

#[derive(Debug)]
pub struct ArgsError(pub Cow<'static, str>);

impl Display for ArgsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ArgsError {}

impl Args {
    pub fn from<I, T>(itr: I) -> Args
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Args::parse_from(itr)
    }

    pub fn mode(&self) -> Result<Mode, ArgsError> {
        match (self.compress, self.expand) {
            (true, true) | (false, false) => Err(ArgsError(Cow::Borrowed(
                "either one of --compress or --expand must be specified",
            ))),
            (true, false) => Ok(Mode::Compress),
            (false, true) => Ok(Mode::Expand),
        }
    }

    pub fn input_reader(&self) -> Result<Box<dyn Read>, Box<dyn Error>> {
        self.input_file
            .as_ref()
            .map_or(Ok(Box::new(io::stdin())), |path| {
                let path = Path::new(&path);
                if !path.exists() {
                    Err(Box::new(ArgsError(Cow::Owned(format!(
                        "failed to open input file {path:?}"
                    )))))
                } else {
                    Ok(Box::new(File::open(path)?))
                }
            })
    }

    pub fn output_writer(&self) -> Result<Box<dyn Write>, io::Error> {
        self.output_file
            .as_ref()
            .map_or(Ok(Box::new(io::stdout())), |path| {
                Ok(Box::new(BufWriter::new(File::create(path)?)))
            })
    }

    pub fn dict_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        match &self.dictionary {
            None => {
                let local_img_path = Path::new("dict.img");
                if local_img_path.exists() {
                    Ok(local_img_path.to_owned())
                } else {
                    let local_txt_path = Path::new("dict.txt");
                    if local_txt_path.exists() {
                        Ok(local_txt_path.to_owned())
                    } else {
                        // resolved to ${HOME} (in *nix-based systems) or %USERPROFILE% in Windows
                        match home::home_dir() {
                            None => {
                                Err(Box::new(ArgsError(Cow::Borrowed("the user's home directory could not be located; please specify the dictionary file"))))
                            }
                            Some(path) => {
                                let home_img_path = path.as_path().join(Path::new("/.serbzip/dict.img"));
                                if home_img_path.exists() {
                                    Ok(home_img_path.to_owned())
                                } else {
                                    Err(Box::new(ArgsError(Cow::Borrowed("no dict.img in ~/.serbzip; please specify the dictionary file"))))
                                }
                            }
                        }
                    }
                }
            }
            Some(path) => {
                let specified_path = Path::new(&path);
                if specified_path.exists() {
                    Ok(specified_path.to_owned())
                } else {
                    Err(Box::new(ArgsError(Cow::Owned(format!(
                        "failed to open dictionary file {specified_path:?}"
                    )))))
                }
            }
        }
    }

    pub fn dictionary_image_output_file(&self) -> &Option<String> {
        &self.dictionary_image_output_file
    }
}
