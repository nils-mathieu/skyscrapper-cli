//! Provides a way to solve Skyscrapper games.

pub enum SolutionError {
    Interrupted,
    NoSolution,
}

pub fn solve(header: &[u8], size: usize) -> Result<Box<[u8]>, SolutionError> {
    let mut solution: Box<[u8]> = (1..=size as u8).cycle().take(size * size).collect();

    // The index of the line that's being computed.
    let mut line = 0;
    while line != size {
        // Look for a valid permutation.
        while !is_line_valid(&header, &solution, size, line) {
            if crate::sigint::occured() {
                return Err(SolutionError::Interrupted);
            }

            let slice = &mut solution[line * size..(line + 1) * size];
            if !next_permutation(slice) {
                // There is no more permutations available for this slice.
                // We have to decrease the index and reset the permutation.
                for i in 0..size {
                    slice[i] = (i + 1) as u8;
                }

                line = line.checked_sub(1).ok_or(SolutionError::NoSolution)?;
            }
        }

        line += 1;
    }

    Ok(solution)
}

/// Computes another permutation for the provided slice.
fn next_permutation(slice: &mut [u8]) -> bool {
    // Find the longest non-increasing suffix.
    let mut pivot = usize::MAX;
    for i in (0..slice.len() - 1).rev() {
        if slice[i] < slice[i + 1] {
            pivot = i;
            break;
        }
    }
    if pivot == usize::MAX {
        return false;
    }

    // Look for the smallest element of the suffix that's greater than the pivot.
    for i in (0..slice.len()).rev() {
        if slice[i] > slice[pivot] {
            slice.swap(i, pivot);
            break;
        }
    }

    // Reverse the new suffix.
    slice[pivot + 1..].reverse();

    true
}

fn is_line_valid(header: &[u8], solution: &[u8], size: usize, line: usize) -> bool {
    let mut count;
    let mut max;

    // Check the views from the left.
    count = 1;
    max = solution[line * size];
    for x in 1..size {
        if solution[line * size + x] > max {
            max = solution[line * size + x];
            count += 1;
        }

        if count > header[2 * size + line] {
            return false;
        }
    }

    if count != header[2 * size + line] {
        return false;
    }

    // Check the views from the right.
    count = 1;
    max = solution[(line + 1) * size - 1];
    for x in (0..size - 1).rev() {
        if solution[line * size + x] > max {
            max = solution[line * size + x];
            count += 1;
        }

        if count > header[3 * size + line] {
            return false;
        }
    }

    if count != header[3 * size + line] {
        return false;
    }

    // Check the views from the top.
    for x in 0..size {
        count = 1;
        max = solution[x];
        for y in 1..=line {
            if solution[x + line * y] > max {
                max = solution[x + line * y];
                count += 1;
            }

            if count > header[x] {
                return false;
            }
        }

        if count + size as u8 - max < header[x] {
            return false;
        }
    }

    if line == size - 1 {
        for x in 0..size {
            count = 1;
            max = solution[size * size - size + x];
            for y in (0..line).rev() {
                if solution[x + line * y] > max {
                    max = solution[x + line * y];
                    count += 1;
                }

                if count > header[size + x] {
                    return false;
                }
            }

            if count != header[size + x] {
                return false;
            }
        }
    }

    true
}
