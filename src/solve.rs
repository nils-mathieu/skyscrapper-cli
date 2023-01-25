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

/// Information about the number of views visible from a given edge of the board.
#[derive(Debug, Clone, Copy)]
struct LineOfSight {
    /// The minimum number of skyscrappers visible from the line of sight.
    pub min: u8,
    /// The maximum number of skyscrappers visible from the line of sight.
    pub max: u8,
}

impl LineOfSight {
    /// Queries information about a line-of-sight.
    ///
    /// The board is expected to contain meaningful values from indices 0 to (an including) `m`.
    pub fn increasing(size: u8, m: u8, mut map: impl FnMut(u8) -> u8) -> Self {
        let mut highest = 0;
        let mut count = 0;

        for n in 0..=m {
            let val = map(n);
            if val > highest {
                highest = val;
                count += 1;
            }
        }

        Self {
            min: count + (highest != size) as u8,
            max: count + size - highest,
        }
    }

    /// Queries information about a life-of-sight.
    ///
    /// The board is expected to contain meaningful values from indices `0` to (and including)
    /// `m`.
    pub fn decreasing(size: u8, m: u8, mut map: impl FnMut(u8) -> u8) -> Self {
        let mut seen = [false; 32];
        let mut count = 0;
        let mut next_highest = size;

        for n in 0..=m {
            let val = map(n);

            seen[val as usize - 1] = true;

            if val == next_highest {
                count += 1;

                while next_highest > 0 && seen[next_highest as usize - 1] {
                    next_highest -= 1;
                }
            }
        }

        let next_highest_candidates = seen[..next_highest as usize]
            .iter()
            .filter(|&&b| !b)
            .count();

        Self {
            min: count,
            max: count + next_highest_candidates as u8,
        }
    }

    /// Returns whether `val` is included within this range.
    #[inline(always)]
    pub fn contains(self, val: u8) -> bool {
        self.min <= val && val <= self.max
    }
}

/// Returns a function that maps an `x` to the line at height `y` in `board`.
fn left_to_right(board: &[u8], size: usize, y: usize) -> impl '_ + Fn(u8) -> u8 {
    move |x| board[x as usize + y * size]
}

/// Returns a function that maps an `x` to the line at height `y` in `board`.
#[cfg(test)]
fn right_to_left(board: &[u8], size: usize, y: usize) -> impl '_ + Fn(u8) -> u8 {
    move |x| board[size - 1 - x as usize + y * size]
}

/// Returns a function that maps a `y` to the column at width `x` in `board`.
fn top_to_bottom(board: &[u8], size: usize, x: usize) -> impl '_ + Fn(u8) -> u8 {
    move |y| board[x + y as usize * size]
}

/// Returns a function that maps a `y` to the column at width `x` in `board`.
#[cfg(test)]
fn bottom_to_top(board: &[u8], size: usize, x: usize) -> impl '_ + Fn(u8) -> u8 {
    move |y| board[x + (size - 1 - y as usize) * size]
}

/// Determines whether the element placed at `index` is valid.
///
/// This function assumes that only elements *before* `index` are initialized.
fn is_board_valid(header: &[u8], board: &[u8], size: usize, index: usize) -> bool {
    let x = index % size;
    let y = index / size;

    let item = board[index];

    if (0..x).any(move |x| board[x + y * size] == item) {
        // Found a double horizontally.
        return false;
    }

    if (0..y).any(move |y| board[x + y * size] == item) {
        // Found a double vertically.
        return false;
    }

    if !LineOfSight::increasing(size as u8, x as u8, left_to_right(board, size, y))
        .contains(header[size * 2 + y])
    {
        // Invalid view count from the left.
        return false;
    }

    if x == size - 1
        && !LineOfSight::decreasing(size as u8, x as u8, left_to_right(board, size, y))
            .contains(header[size * 3 + y])
    {
        // Invalid view count from the right.
        return false;
    }

    if !LineOfSight::increasing(size as u8, y as u8, top_to_bottom(board, size, x))
        .contains(header[x])
    {
        // Invalid view count from the top.
        return false;
    }

    if y == size - 1
        && !LineOfSight::decreasing(size as u8, y as u8, top_to_bottom(board, size, x))
            .contains(header[size + x])
    {
        // Invalid view count from the bottom.
        return false;
    }

    // if !LineOfSight::decreasing(x as u8, move |t| result[t as usize + y * size])
    //     .contains(header[size * 3 + y])
    // {
    //     // Invalid view count from the right.
    //     return false;
    // }

    // if !LineOfSight::decreasing(y as u8, move |t| result[x + t as usize * size])
    //     .contains(header[size + x])
    // {
    //     // Invalid view count from the bottom.
    //     return false;
    // }

    true
}

#[test]
#[cfg(test)]
fn increasing_line_of_sight_one() {
    let los = LineOfSight::increasing(5, 2, |i| [1, 3, 2][i as usize]);
    assert_eq!(los.min, 3);
    assert_eq!(los.max, 4);
}

#[test]
#[cfg(test)]
fn increasing_line_of_sight_full() {
    let los = LineOfSight::increasing(5, 4, |i| [1, 3, 2, 5, 4][i as usize]);
    assert_eq!(los.min, 3);
    assert_eq!(los.max, 3);
}

#[test]
#[cfg(test)]
fn line_of_sight_basic_board() {
    /*
      2 2 1 2
    2 2 1 4 3 2
    2 1 4 3 2 3
    1 4 3 2 1 4
    2 3 2 1 4 1
      2 3 4 1
    */

    #[rustfmt::skip]
    let test_board = &[
        2, 1, 4, 3,
        1, 4, 3, 2,
        4, 3, 2, 1,
        3, 2, 1, 4,
    ];

    let header = [2, 2, 1, 2, 2, 3, 4, 1, 2, 2, 1, 2, 2, 3, 4, 1];

    // ROWS

    let los = LineOfSight::increasing(4, 3, left_to_right(test_board, 4, 0));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[4 * 2], 2);

    let los = LineOfSight::increasing(4, 3, right_to_left(test_board, 4, 0));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[4 * 3], 2);

    let los = LineOfSight::increasing(4, 3, left_to_right(test_board, 4, 1));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[4 * 2 + 1], 2);

    let los = LineOfSight::increasing(4, 3, right_to_left(test_board, 4, 1));
    assert_eq!(los.min, 3);
    assert_eq!(los.max, 3);
    assert_eq!(header[4 * 3 + 1], 3);

    let los = LineOfSight::increasing(4, 3, left_to_right(test_board, 4, 2));
    assert_eq!(los.min, 1);
    assert_eq!(los.max, 1);
    assert_eq!(header[4 * 2 + 2], 1);

    let los = LineOfSight::increasing(4, 3, right_to_left(test_board, 4, 2));
    assert_eq!(los.min, 4);
    assert_eq!(los.max, 4);
    assert_eq!(header[4 * 3 + 2], 4);

    let los = LineOfSight::increasing(4, 3, left_to_right(test_board, 4, 3));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[4 * 2 + 3], 2);

    let los = LineOfSight::increasing(4, 3, right_to_left(test_board, 4, 3));
    assert_eq!(los.min, 1);
    assert_eq!(los.max, 1);
    assert_eq!(header[4 * 3 + 3], 1);

    // COLUMNS

    let los = LineOfSight::increasing(4, 3, top_to_bottom(test_board, 4, 0));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[0], 2);

    let los = LineOfSight::increasing(4, 3, bottom_to_top(test_board, 4, 0));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[4], 2);

    let los = LineOfSight::increasing(4, 3, top_to_bottom(test_board, 4, 1));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[1], 2);

    let los = LineOfSight::increasing(4, 3, bottom_to_top(test_board, 4, 1));
    assert_eq!(los.min, 3);
    assert_eq!(los.max, 3);
    assert_eq!(header[4 + 1], 3);

    let los = LineOfSight::increasing(4, 3, top_to_bottom(test_board, 4, 2));
    assert_eq!(los.min, 1);
    assert_eq!(los.max, 1);
    assert_eq!(header[2], 1);

    let los = LineOfSight::increasing(4, 3, bottom_to_top(test_board, 4, 2));
    assert_eq!(los.min, 4);
    assert_eq!(los.max, 4);
    assert_eq!(header[4 + 2], 4);

    let los = LineOfSight::increasing(4, 3, top_to_bottom(test_board, 4, 3));
    assert_eq!(los.min, 2);
    assert_eq!(los.max, 2);
    assert_eq!(header[3], 2);

    let los = LineOfSight::increasing(4, 3, bottom_to_top(test_board, 4, 3));
    assert_eq!(los.min, 1);
    assert_eq!(los.max, 1);
    assert_eq!(header[4 + 3], 1);
}

#[test]
#[cfg(test)]
fn line_of_sight_contains() {
    let los = LineOfSight { min: 1, max: 3 };

    assert!(!los.contains(0));
    assert!(los.contains(1));
    assert!(los.contains(2));
    assert!(los.contains(3));
    assert!(!los.contains(4));
    assert!(!los.contains(5));
}

#[test]
#[cfg(test)]
fn line_of_sight_contains_one() {
    let los = LineOfSight { min: 2, max: 2 };

    assert!(los.contains(2));
    assert!(!los.contains(1));
    assert!(!los.contains(3));
}

#[test]
#[cfg(test)]
fn left_to_right_fn() {
    let board = [1, 2, 3, 4, 5];
    let f = left_to_right(&board, 5, 0);

    assert_eq!(f(0), 1);
    assert_eq!(f(1), 2);
    assert_eq!(f(2), 3);
    assert_eq!(f(3), 4);
    assert_eq!(f(4), 5);
}

#[test]
#[cfg(test)]
fn right_to_left_fn() {
    let board = [1, 2, 3, 4, 5];
    let f = right_to_left(&board, 5, 0);

    assert_eq!(f(0), 5);
    assert_eq!(f(1), 4);
    assert_eq!(f(2), 3);
    assert_eq!(f(3), 2);
    assert_eq!(f(4), 1);
}

#[test]
#[cfg(test)]
fn top_to_bottom_fn() {
    #[rustfmt::skip]
    let board = [
        1, 2, 3, 4, 5,
        1, 3, 3, 4, 5,
        1, 5, 3, 4, 5,
        1, 6, 3, 4, 5,
        1, 7, 3, 4, 5,
    ];
    let f = top_to_bottom(&board, 5, 1);

    assert_eq!(f(0), 2);
    assert_eq!(f(1), 3);
    assert_eq!(f(2), 5);
    assert_eq!(f(3), 6);
    assert_eq!(f(4), 7);
}

#[test]
#[cfg(test)]
fn bottom_to_top_fn() {
    #[rustfmt::skip]
    let board = [
        1, 2, 3, 4, 5,
        1, 3, 3, 4, 5,
        1, 5, 3, 4, 5,
        1, 6, 3, 4, 5,
        1, 7, 3, 4, 5,
    ];
    let f = bottom_to_top(&board, 5, 1);

    assert_eq!(f(0), 7);
    assert_eq!(f(1), 6);
    assert_eq!(f(2), 5);
    assert_eq!(f(3), 3);
    assert_eq!(f(4), 2);
}
