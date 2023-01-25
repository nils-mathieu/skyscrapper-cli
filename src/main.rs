#![allow(clippy::write_with_newline)]

use std::fmt::Display;
use std::io;
use std::io::Write;
use std::process::ExitCode;

use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

mod args;
mod generate;
mod solve;

mod sigint;

/// The glorious entry point.
fn main() -> ExitCode {
    sigint::initialize();
    let args = args::parse();

    let color_choice = if atty::is(atty::Stream::Stdout) {
        termcolor::ColorChoice::Auto
    } else {
        termcolor::ColorChoice::Never
    };

    match args.command {
        args::Command::Generate { output, seed, size } => {
            if size == 0 {
                return ExitCode::from(3);
            }

            // Setup a random number generator.
            // If the user provided a set seed, create the pRNG with it, otherwise generate a
            // random seed.
            let mut rng = match seed {
                Some(seed) => Xoroshiro128StarStar::seed_from_u64(seed),
                None => Xoroshiro128StarStar::from_entropy(),
            };

            // Generate the solution.
            let solution = match generate::generate_solution(&mut rng, size) {
                Some(s) => s,
                // The operation has been interrupted by a CTRL+C.
                None => return ExitCode::SUCCESS,
            };

            // Open the standard output.
            let stdout = termcolor::StandardStream::stdout(color_choice);
            let mut stdout = stdout.lock();

            // If no output has been specified, use the `OutputFormat::Both` format.
            if output.is_empty() {
                let _ = print_solution(&mut stdout, &solution, size, &args::OutputFormat::Both);
            } else {
                let mut iter = output.iter();

                if let Some(first) = iter.next() {
                    let _ = print_solution(&mut stdout, &solution, size, first);
                }

                for output in iter {
                    let _ = stdout.write_all(b"\n");
                    let _ = print_solution(&mut stdout, &solution, size, output);
                }
            }

            ExitCode::SUCCESS
        }
        args::Command::Solve { header, output } => {
            let size = header.0.len() / 4;

            if size == 0 {
                return ExitCode::from(3);
            }

            let solution = match solve::solve(&header.0, size) {
                Ok(ok) => ok,
                Err(solve::SolutionError::Interrupted) => return ExitCode::SUCCESS,
                Err(solve::SolutionError::NoSolution) => {
                    use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

                    let stderr = StandardStream::stderr(color_choice);
                    let mut stderr = stderr.lock();

                    let _ = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                    let _ = write!(stderr, "error");
                    let _ = stderr.reset();
                    let _ = writeln!(stderr, ": no solution found");

                    return ExitCode::FAILURE;
                }
            };

            let stdout = termcolor::StandardStream::stdout(color_choice);
            let mut stdout = stdout.lock();

            let _ = print_solution(&mut stdout, &solution, size as u8, &output);

            ExitCode::SUCCESS
        }
    }
}

fn log10(mut size: u8) -> usize {
    let mut log10 = 0;
    while size != 0 {
        size /= 10;
        log10 += 1;
    }
    log10
}

/// Prints the provided solution according to the provided output format.
fn print_solution(
    w: &mut dyn termcolor::WriteColor,
    solution: &[u8],
    size: u8,
    output: &args::OutputFormat,
) -> io::Result<()> {
    match output {
        args::OutputFormat::Solution => {
            w.set_color(
                termcolor::ColorSpec::new()
                    .set_fg(Some(termcolor::Color::Blue))
                    .set_intense(true),
            )?;
            for chunk in solution.chunks_exact(size as usize) {
                print_iterator(w, chunk, log10(size))?;
                w.write_all(b"\n")?;
            }
            w.reset()?;
        }
        args::OutputFormat::HeaderLine => {
            let header = generate::solution_to_header(&solution, size);
            w.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
            print_iterator(w, header.as_ref(), 0)?;
            w.reset()?;
            w.write_all(b"\n")?;
        }
        args::OutputFormat::Header => {
            print_both(w, &solution, size, false)?;
        }
        args::OutputFormat::Both => {
            print_both(w, &solution, size, true)?;
        }
    }

    Ok(())
}

/// Writes the elements of the provided iterator to the standard output. Each element is separated
/// by exactly `max_len + 1` spaces.
fn print_iterator<I: IntoIterator>(
    w: &mut dyn termcolor::WriteColor,
    it: I,
    max_len: usize,
) -> io::Result<()>
where
    I::Item: Display,
{
    let mut it = it.into_iter();

    if let Some(first) = it.next() {
        write!(w, "{first:<max_len$}")?;
    }

    for k in it {
        write!(w, " {k:<max_len$}")?;
    }

    Ok(())
}

/// Prints both the header and the solution together.
fn print_both(
    mut w: &mut dyn termcolor::WriteColor,
    solution: &[u8],
    size: u8,
    actually_display_solution: bool,
) -> io::Result<()> {
    let s = size as usize;
    let size_len = log10(size);

    let header = generate::solution_to_header(&solution, size);

    // First Line
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    w.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
    print_iterator(&mut w, &header[0..s], size_len)?;
    w.reset()?;
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    w.write_all(b"\n")?;

    // Middle Lines
    for (i, chunk) in solution.chunks_exact(s).enumerate() {
        w.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
        write!(w, "{:<size_len$} ", header[2 * s + i])?;
        w.reset()?;

        if actually_display_solution {
            w.set_color(
                termcolor::ColorSpec::new()
                    .set_fg(Some(termcolor::Color::Blue))
                    .set_intense(true),
            )?;
            print_iterator(w, chunk, size_len)?;
            w.reset()?;
        } else {
            for _ in 0..s * (size_len + 1) - 1 {
                w.write_all(b" ")?;
            }
        }

        w.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
        write!(w, " {:<size_len$}\n", header[3 * s + i])?;
        w.reset()?;
    }

    // Last Line
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    w.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
    print_iterator(w, &header[s..2 * s], size_len)?;
    w.reset()?;
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    w.write_all(b"\n")?;

    Ok(())
}
