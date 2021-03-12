#![deny(warnings)]
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

use structopt::StructOpt;
use thiserror::Error;

use crate::ezconsole::style_e;
use crate::formatter::Formatter;

mod ezconsole;
mod formatter;

/// A cursed formatting tool.
///
/// Transforms C-style code into a more Pythonic format.
/// Files are formatted in-place, though a temporary file is used to prevent partial writes.
///
/// Providing no files will result in reading from standard input and writing to standard output.
///
/// Assumptions:
///
/// - Your code is already formatted well. This tool does not re-format indentation to match Python.
#[derive(StructOpt)]
#[structopt(name = "pythonicfmt")]
struct PythonicFormat {
    /// The input path(s), can be files or directories
    ///
    /// Directories will be searched recursively.
    #[structopt(parse(from_os_str))]
    input: Vec<PathBuf>,
    /// The column to start storing "junk" (semi-colons, braces) at
    #[structopt(long, default_value = "120")]
    junk_column: usize,
}

impl From<&PythonicFormat> for Formatter {
    fn from(args: &PythonicFormat) -> Self {
        let mut formatter = Formatter::default();
        formatter.junk_column(args.junk_column);
        formatter
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("I/O Error occurred: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Formatter error: {0:?}")]
    FormatterError(#[from] formatter::Error),
}

type Result<T> = std::result::Result<T, Error>;

fn main() {
    let args: PythonicFormat = PythonicFormat::from_args();
    if let Err(error) = main_for_result(args) {
        eprintln!("{}", style_e(format!("Error: {:?}", error)).red());
        exit(1);
    }
}

fn main_for_result(args: PythonicFormat) -> Result<()> {
    let formatter = Formatter::from(&args);
    let mut any_files = false;
    for file_res in flatten_files(args.input) {
        any_files = true;
        let file = file_res?;
        eprintln!("Formatting {}", file.display());
        let temporary = tempfile::NamedTempFile::new_in(file.parent().expect("No parent dir?"))?;
        let pipe_in = std::fs::File::open(&file)?;
        process_pipe(&formatter, pipe_in, temporary.as_file())?;
        temporary.persist(&file).map_err(|e| e.error)?;
    }
    if !any_files {
        eprintln!("Formatting standard input to standard output");
        process_pipe(&formatter, std::io::stdin(), std::io::stdout())?;
    }

    Ok(())
}

fn flatten_files(files: Vec<PathBuf>) -> impl Iterator<Item = std::io::Result<PathBuf>> {
    files.into_iter().flat_map(
        |file| -> Box<dyn Iterator<Item = std::io::Result<PathBuf>>> {
            if file.is_dir() {
                Box::new(
                    walkdir::WalkDir::new(file)
                        .into_iter()
                        .filter_entry(|e| e.file_type().is_file())
                        .map(|r| match r {
                            Ok(d) => Ok(d.into_path()),
                            Err(e) => Err(std::io::Error::from(e)),
                        }),
                )
            } else {
                Box::new(vec![Ok(file)].into_iter())
            }
        },
    )
}

fn process_pipe(
    formatter: &Formatter,
    mut pipe_in: impl Read,
    mut pipe_out: impl Write,
) -> Result<()> {
    let mut content = String::new();
    pipe_in.read_to_string(&mut content)?;
    formatter.format(&mut content)?;
    pipe_out.write(content.as_bytes())?;
    Ok(())
}
