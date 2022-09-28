use std::fmt::Display;
use std::io;
use std::io::Write;

use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

pub mod args;
pub mod generate;

mod sigint;

/// The glorious entry point.
fn main() {
    sigint::initialize();
    let args = args::parse();

    match args.command {
        args::Command::Generate { output, seed, size } => {
            if size == 0 {
                let _ = std::io::stdout().write_all(b"\n");
                return;
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
                None => return,
            };

            // Open the standard output.
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            // Depending on the requested output, print the solution in a certain way.
            match output {
                args::GenerateOutput::Solution => {
                    for chunk in solution.chunks_exact(size as usize) {
                        let _ = print_iterator(&mut stdout, chunk, log10(size));
                        let _ = stdout.write_all(b"\n");
                    }
                }
                args::GenerateOutput::HeaderLine => {
                    let header = generate::solution_to_header(&solution, size);
                    let _ = print_iterator(&mut stdout, header.as_ref(), 0);
                }
                args::GenerateOutput::Header => {
                    let _ = print_both(&mut stdout, &solution, size, false);
                }
                args::GenerateOutput::Both => {
                    let _ = print_both(&mut stdout, &solution, size, true);
                }
            }
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

/// Writes the elements of the provided iterator to the standard output. Each element is separated
/// by exactly `max_len + 1` spaces.
fn print_iterator<I: IntoIterator>(w: &mut dyn io::Write, it: I, max_len: usize) -> io::Result<()>
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
    mut w: &mut dyn io::Write,
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
    print_iterator(&mut w, &header[0..s], size_len)?;
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    w.write_all(b"\n")?;

    // Middle Lines
    for (i, chunk) in solution.chunks_exact(s).enumerate() {
        write!(w, "{:<size_len$} ", header[2 * s + i])?;

        if actually_display_solution {
            print_iterator(w, chunk, size_len)?;
        } else {
            for _ in 0..s * (size_len + 1) - 1 {
                w.write_all(b" ")?;
            }
        }

        write!(w, " {:<size_len$}\n", header[3 * s + i])?;
    }

    // Last Line
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    print_iterator(w, &header[s..2 * s], size_len)?;
    for _ in 0..size_len + 1 {
        w.write_all(b" ")?;
    }
    w.write_all(b"\n")?;

    Ok(())
}
