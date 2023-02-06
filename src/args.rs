//! Defines the [`Args`] structure.

use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};

/// A CLI tool to play the Skyscrapper game.
#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// The selected subcommand.
    #[clap(subcommand)]
    pub command: Command,
}

/// The output type of the [`Command::Generate`] subcommand.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Only print the solution.
    Solution,
    /// Only print the header.
    Header,
    /// Only print the header, on one single line.
    HeaderLine,
    /// Print both the header and the solution.
    Both,
}

/// A possible command for the CLI tool.
#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Generate a random Skyscrapper header.
    Generate {
        /// Whether the solution should be displayed rather than the header.
        #[clap(long, short = 'o', value_enum)]
        output: Vec<OutputFormat>,
        /// Provides the seed that should be used to generate the board.
        #[clap(long)]
        seed: Option<u64>,
        /// The size of the board.
        size: u8,
    },
    /// Solves a board given a specific header.
    ///
    /// The header must be provided using the same format as the one outputed by header-line.
    Solve {
        /// The header that will be solved.
        header: Header,
        /// Whether the process should be animated.
        #[clap(long, short, action)]
        animate: bool,
        /// The generated output.
        #[clap(long, short = 'o', value_enum, default_value_t = OutputFormat::Both)]
        output: OutputFormat,
    },
    /// Determines whether a given response is valid.
    ///
    /// This command expects the board to be provided without its header in its standard input.
    Check {
        /// The header that the board will be verified against.
        header: Header,
    },
}

/// An error that might occur whilst parsing a [`Header`] instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseHeaderError {
    InvalidInteger,
    InvalidViewCount,
    TooManyViews,
    ViewTooLarge,
    ViewZero,
}

impl From<std::num::ParseIntError> for ParseHeaderError {
    fn from(e: std::num::ParseIntError) -> Self {
        use std::num::IntErrorKind::*;

        if *e.kind() == PosOverflow {
            Self::ViewTooLarge
        } else {
            Self::InvalidInteger
        }
    }
}

impl Display for ParseHeaderError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidInteger => f.write_str("invalid integer found in header"),
            Self::InvalidViewCount => f.write_str("invalid number of views (must be a multiple of 4)"),
            Self::TooManyViews => f.write_str("it's not possible to solve a size larger than 255"),
            Self::ViewTooLarge => f.write_str("views can't exceed the size of the board"),
            Self::ViewZero => f.write_str("views can't be 0"),
        }
    }
}

impl std::error::Error for ParseHeaderError {}

/// A simple wrapper around [`Box<[u8]>`] that gets parsed like a skyscrapper header line through
/// its [`FromStr`] implementation.
#[derive(Clone, Debug)]
pub struct Header(pub Box<[u8]>);

// A string representing a "header" must follow the following properties:
//
// It's a space-separated list of numbers. The number of elements in that list must be divisible
// by 4.
//
// Let call "n" the quarter of that size. Each element of the list must be between 1 and n
// (included). n must fit in a u8.
impl FromStr for Header {
    type Err = ParseHeaderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut vec = Vec::new();

        // FIXME(nils): use try_collect() when stable.
        for word in s.split_ascii_whitespace() {
            let view = word.parse()?;
            if view == 0 {
                return Err(ParseHeaderError::ViewZero);
            }
            vec.push(view);
        }

        if vec.len() % 4 != 0 {
            return Err(ParseHeaderError::InvalidViewCount);
        }

        if vec.len() > 255 * 4 {
            return Err(ParseHeaderError::TooManyViews);
        }

        let size = (vec.len() / 4) as u8;

        if vec.iter().any(|&v| v > size) {
            return Err(ParseHeaderError::ViewTooLarge);
        }

        Ok(Header(vec.into_boxed_slice()))
    }
}

/// Parses the arguments passed to the program and parses then into an instance of [`Args`]. If an
/// error occurs, the program exits.
///
/// # Notes
///
/// In case of error, the values currently leaving on the stack will *not* be dropped.
pub fn parse() -> Args {
    match Parser::try_parse() {
        Ok(ok) => ok,
        Err(err) => {
            // If an error occur whilst printing, there is not much we can do about it.
            let _ = err.print();
            std::process::exit(2);
        }
    }
}
