//! Board formatting.

use std::fmt::Display;
use std::io;

use crate::args;
use crate::generate;

fn log10(mut size: u8) -> usize {
    let mut log10 = 0;
    while size != 0 {
        size /= 10;
        log10 += 1;
    }
    log10
}

/// Prints the provided solution according to the provided output format.
pub fn print_solution(
    w: &mut dyn termcolor::WriteColor,
    solution: &[u8],
    header: &[u8],
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
            let header = generate::solution_to_header(solution, size);
            w.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)))?;
            print_iterator(w, header.as_ref(), 0)?;
            w.reset()?;
            w.write_all(b"\n")?;
        }
        args::OutputFormat::Header => {
            print_both(w, solution, header, size, false)?;
        }
        args::OutputFormat::Both => {
            print_both(w, solution, header, size, true)?;
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
pub fn print_both(
    mut w: &mut dyn termcolor::WriteColor,
    solution: &[u8],
    header: &[u8],
    size: u8,
    actually_display_solution: bool,
) -> io::Result<()> {
    let s = size as usize;
    let size_len = log10(size);

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
