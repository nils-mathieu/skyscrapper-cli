//! Provides a way to solve Skyscrapper games.

pub enum SolutionError {
    Interrupted,
    NoSolution,
}

pub fn solve(header: &[u8], size: usize) -> Result<Box<[u8]>, SolutionError> {
    // This is the result of the operation.
    //
    // It is also used used to keep track of which numbers were already tested.
    let mut result = vec![1u8; size * size].into_boxed_slice();

    // The index of the search. Every item *before* that index is fixed, and every element *after*
    // that index is yet to be tested.
    let mut index = 0;

    loop {
        if crate::sigint::occured() {
            return Err(SolutionError::Interrupted);
        }

        if is_board_valid(header, &result, size, index) {
            // It works!
            // Let's continue with the next index.
            index += 1;

            // We're out of bounds, meaning that the solution is complete!
            if index >= result.len() {
                break;
            }
        } else {
            // It does not work.
            // We need to try something else.
            result[index] += 1;

            while result[index] > size as u8 {
                // We're out of options for this index. We have to backtrack.
                result[index] = 1u8;

                if index == 0 {
                    // We're already at the begining!
                    // There is no solution for that header.
                    return Err(SolutionError::NoSolution);
                }

                index -= 1;
                result[index] += 1;
            }
        }
    }

    Ok(result)
}

fn count_views(size: usize, map: &mut dyn FnMut(usize) -> u8) -> u8 {
    let mut max_so_far = 0;
    let mut views = 0;

    for i in 0..size {
        let val = map(i);
        if val > max_so_far {
            max_so_far = val;
            views += 1;
        }
    }

    views
}

/// Determines whether the element placed at `index` is valid.
///
/// This function assumes that only elements *before* `index` are initialized.
fn is_board_valid(header: &[u8], result: &[u8], size: usize, index: usize) -> bool {
    let x = index % size;
    let y = index / size;

    let item = result[index];

    if (0..x).any(move |x| result[x + y * size] == item) {
        // found double horizontally
        return false;
    }

    if (0..y).any(move |y| result[x + y * size] == item) {
        // found double vertically
        return false;
    }

    if x == size - 1 {
        if count_views(size, &mut move |x| result[x + y * size]) != header[size * 2 + y] {
            // invalid view count from the left
            return false;
        }

        if count_views(size, &mut move |x| result[size - x - 1 + y * size]) != header[size * 3 + y]
        {
            // invalid view count from the right
            return false;
        }
    }

    if y == size - 1 {
        if count_views(size, &mut move |y| result[x + y * size]) != header[x] {
            // invalid view count from the top
            return false;
        }

        if count_views(size, &mut move |y| result[x + (size - y - 1) * size]) != header[size + x] {
            // invalid view count from the bottom
            return false;
        }
    }

    true
}
