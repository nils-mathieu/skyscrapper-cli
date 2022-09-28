//! Implements functionalities for the `generate` subcommand.

use rand::{Rng, RngCore};

/// Generates a random Skyscrapper solution.
///
/// `None` is returned when the operation has been interrupted.
pub fn generate_solution(rng: &mut dyn RngCore, size: u8) -> Option<Box<[u8]>> {
    let size = size as usize;

    // The solution that's being created.
    let mut solution: Box<[u8]> = std::iter::repeat(0).take(size * size).collect();

    // A simple stack that keeps track of which numbers can be added at a specific position.
    let mut stack: Vec<u8> = Vec::new();
    // This vector contains the starting index of every slice stored in `stack`.
    let mut stack_slices: Vec<usize> = Vec::new();

    //
    //                numbers for|numbers for
    //                first index|second index
    //               +-----------+----
    //         stack | 0 1 3 4 5 | 1 3
    //               +-----------+----
    //               +-----------+----
    //  stack_slices | 0         | 5
    //               +-----------+----
    //

    // The index for which we are computing a value.
    let mut index = 0;

    while index != size * size {
        if crate::sigint::occured() {
            return None;
        }

        // Compute the numbers available for the next slice.
        let x = index % size;
        let y = index / size;
        stack_slices.push(stack.len());
        stack.extend(
            (1..=size as u8)
                .filter(|&c| (0..x).all(|i| solution[i + y * size] != c))
                .filter(|&c| (0..y).all(|i| solution[x + i * size] != c)),
        );

        while stack.len() == *stack_slices.last().unwrap() {
            // The next number cannot be generated: there is no valid value.
            // In that case, we have to backtrack (or retry, if only one stacked slice is empty).

            index -= 1;
            solution[index] = 0;
            stack_slices.pop();
        }

        // Choose a number on the top of the stack.
        let choosen_index = rng.gen_range(*stack_slices.last().unwrap()..stack.len());
        solution[index] = stack.swap_remove(choosen_index);
        index += 1;
    }

    Some(solution)
}

fn count_viewed(size: u8, get_number: &mut dyn FnMut(usize) -> u8) -> u8 {
    let mut max = 0;
    let mut count = 0;

    for i in 0..size as usize {
        let n = get_number(i);
        if n > max {
            max = n;
            count += 1;

            if max == size {
                break;
            }
        }
    }

    count
}

/// Converts an existing Skyscrapper solution into a Skyscrapper header.
pub fn solution_to_header(solution: &[u8], size: u8) -> Box<[u8]> {
    let s = size as usize;

    let mut header: Box<[u8]> = std::iter::repeat(0).take(s * 4).collect();

    // Up
    for x in 0..size as usize {
        header[x] = count_viewed(size, &mut |i| solution[x + i * s]);
    }
    // Down
    for x in 0..size as usize {
        header[s + x] = count_viewed(size, &mut |i| solution[x + (s - i - 1) * s]);
    }
    // Left
    for y in 0..size as usize {
        header[2 * s + y] = count_viewed(size, &mut |i| solution[i + y * s]);
    }
    // Right
    for y in 0..size as usize {
        header[3 * s + y] = count_viewed(size, &mut |i| solution[s - i - 1 + y * s]);
    }

    header
}
