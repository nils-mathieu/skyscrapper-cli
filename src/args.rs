/// Defines the [`Args`] structure.
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
    /// Print both.
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
