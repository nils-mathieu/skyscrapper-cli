#![allow(clippy::write_with_newline)]

use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;

use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

mod args;
mod format;
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

            let header = generate::solution_to_header(&solution, size);

            // Open the standard output.
            let stdout = termcolor::StandardStream::stdout(color_choice);
            let mut stdout = stdout.lock();

            // If no output has been specified, use the `OutputFormat::Both` format.
            if output.is_empty() {
                let _ = format::print_solution(
                    &mut stdout,
                    &solution,
                    &header,
                    size,
                    &args::OutputFormat::Both,
                );
            } else {
                let mut iter = output.iter();

                if let Some(first) = iter.next() {
                    let _ = format::print_solution(&mut stdout, &solution, &header, size, first);
                }

                for output in iter {
                    let _ = stdout.write_all(b"\n");
                    let _ = format::print_solution(&mut stdout, &solution, &header, size, output);
                }
            }

            ExitCode::SUCCESS
        }
        args::Command::Solve {
            header,
            output,
            animate,
        } => {
            let size = header.0.len() / 4;

            if size == 0 {
                return ExitCode::from(3);
            }

            let stdout = termcolor::StandardStream::stdout(color_choice);
            let mut stdout = stdout.lock();

            let res = if animate {
                solve::solve_animated(&header.0, size, &mut stdout, Duration::from_millis(20))
            } else {
                solve::solve(&header.0, size)
            };

            let solution = match res {
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

            let _ = format::print_solution(&mut stdout, &solution, &header.0, size as u8, &output);

            ExitCode::SUCCESS
        }
    }
}
