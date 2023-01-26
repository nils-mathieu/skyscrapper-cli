//! Provides a way to solve Skyscrapper games.

use std::time::Duration;

use termcolor::WriteColor;

use crate::format;

pub enum SolutionError {
    Interrupted,
    NoSolution,
}

/// Keeps the state of a line of sight.
///
/// New skyscrappers are expected to be added from the nearest position to the farthest position.
struct PushingLineOfSight {
    /// The expected number of line of sights for this element.
    expected: u8,
    /// The maximums found so far.
    ///
    /// In each tuple, the first element is the actual maximum, and the second element is the
    /// number of elements "hidden" by this maximum (since the previous maximum).
    maximums: Vec<(u8, u8)>,
}

impl PushingLineOfSight {
    /// Creates a new [`PushingLineOfSight`].
    pub fn empty(expected: u8, size: usize) -> Self {
        Self {
            expected,
            maximums: Vec::with_capacity(size),
        }
    }

    /// Returns whether adding a skyscrapper of height `height` into this line of sight would
    /// invalidate its requirements.
    pub fn can_push(&self, size: u8, height: u8) -> bool {
        let highest = match self.maximums.last() {
            Some(&(highest_so_far, _)) => highest_so_far.max(height),
            None => height,
        };

        let num_maxs = self.maximums.len() as u8 + (highest == height) as u8;
        let min = num_maxs + (highest != size) as u8;
        let max = num_maxs + size - highest;
        min <= self.expected && self.expected <= max
    }

    /// Indicates that a new value has been added to the line of sight.
    ///
    /// This function assumes that the validity of the line of sight is preserved.
    pub fn push(&mut self, height: u8) {
        match self.maximums.last_mut() {
            Some((highest_so_far, hidden)) => {
                if *highest_so_far < height {
                    self.maximums.push((height, 0));
                } else {
                    *hidden += 1;
                }
            }
            None => {
                self.maximums.push((height, 0));
            }
        }
    }

    /// Pops the last value of this line-of-sight.
    pub fn pop(&mut self) {
        match self.maximums.last_mut() {
            Some((_highest, hidden)) => {
                if *hidden == 0 {
                    self.maximums.pop();
                } else {
                    *hidden -= 1;
                }
            }
            None => panic!("called `pop` when there was nothing to remove"),
        }
    }
}

/// Keeps the state of a line of sight.
///
/// New skyscrappers are expected to be added from the farthest position to the nearest position.
pub struct PullingLineOfSight {
    /// The number of views expected for this line of sight.
    expected: u8,
    /// The next skyscrapper height that this line of sight will see.
    ///
    /// The most up-to-date is the last one.
    ///
    /// The first element of each tuple is the actual value, and the second one is the number of
    /// values that *will* be hidden by it once it is found.
    next_highest: Vec<(u8, u8)>,
    /// The remaining number of skyscrapper missing from this line of sight.
    remaining: u8,
}

impl PullingLineOfSight {
    /// Creates a new [`PullingLineOfSight`].
    pub fn empty(expected: u8, size: usize) -> Self {
        let mut next_highest = Vec::with_capacity(size);
        next_highest.push((size as u8, 0));

        Self {
            expected,
            next_highest,
            remaining: size as u8,
        }
    }

    /// Determines whether adding a new skyscrapper of height `height` would retain the validity
    /// of this line of sight.
    pub fn can_push(&self, height: u8) -> bool {
        let mut maxs = self.next_highest.len() as u8 - 1;
        maxs += (height == self.next_highest.last().unwrap().0) as u8;

        let min = maxs + (self.remaining != 1) as u8;
        let max = maxs + self.remaining - 1;
        min <= self.expected && self.expected <= max
    }

    pub fn push(&mut self, seen: &[bool], height: u8) {
        self.remaining -= 1;
        let (next, hidden) = self.next_highest.last_mut().unwrap();
        let mut next = *next;
        if next == height {
            loop {
                next -= 1;
                if next == 0 || !seen[next as usize - 1] {
                    break;
                }
            }
            self.next_highest.push((next, 0));
        } else {
            *hidden += 1;
        }
    }

    pub fn pop(&mut self) {
        self.remaining += 1;
        let (_next, hidden) = self.next_highest.last_mut().unwrap();
        if *hidden == 0 {
            self.next_highest.pop();
        } else {
            *hidden -= 1;
        }
    }
}

/// Stores the state of the board
struct Board {
    /// The actual output board.
    board: Box<[u8]>,
    /// The size of the board.
    ///
    /// `board` is a `size * size` array. `header` is a `size * 4` array.
    size: usize,
    /// For each line, then for each column, indicates which skyscrapper height is already present
    /// on the board, and which is not.
    ///
    /// line0, line1, .., column0, column1, ..
    height_presence: Box<[bool]>,
    /// The "pushing" line of sights. top-to-botom, then left-to-right.
    pushing_los: Box<[PushingLineOfSight]>,
    /// The "pulling" line of sights. bottom-to-top, then right-to-left.
    pulling_los: Box<[PullingLineOfSight]>,
    /// The number of skyscrapper that were properly placed on the board.
    index: usize,
    /// The cached result of `index % size`.
    x: usize,
    /// The cached result of `index / size`.
    y: usize,
    /// The next potential candidate to be placed at `index`.
    next_candidate: u8,
}

impl Board {
    /// Creates a new, empty, [`Board`] instance.
    pub fn empty(header: &[u8], size: usize) -> Self {
        Self {
            size,
            board: std::iter::repeat(0u8).take(size * size).collect(),
            height_presence: std::iter::repeat(false).take(size * size * 2).collect(),
            index: 0,
            pushing_los: (0..size)
                .map(move |x| PushingLineOfSight::empty(header[x], size))
                .chain(
                    (0..size).map(move |y| PushingLineOfSight::empty(header[size * 2 + y], size)),
                )
                .collect(),
            pulling_los: (0..size)
                .map(move |x| PullingLineOfSight::empty(header[size + x], size))
                .chain(
                    (0..size).map(move |y| PullingLineOfSight::empty(header[size * 3 + y], size)),
                )
                .collect(),
            x: 0,
            y: 0,
            next_candidate: 1,
        }
    }

    /// Returns whether the board is complete.
    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        self.index == self.board.len()
    }

    /// Returns whether `height` is present on the given line.
    #[inline]
    pub fn is_height_on_line(&self, height: u8, line: usize) -> bool {
        self.height_presence[line * self.size + height as usize - 1]
    }

    /// Sets whether `height` is present on the given line.
    #[inline]
    pub fn set_height_on_line(&mut self, height: u8, line: usize, yes: bool) {
        self.height_presence[line * self.size + height as usize - 1] = yes;
    }

    /// Returns whether `height` is present on the given column.
    #[inline]
    pub fn is_height_on_column(&self, height: u8, column: usize) -> bool {
        self.height_presence[self.size * self.size + column * self.size + height as usize - 1]
    }

    /// Sets whether `height` is present on the given column.
    #[inline]
    pub fn set_height_on_column(&mut self, height: u8, column: usize, yes: bool) {
        self.height_presence[self.size * self.size + column * self.size + height as usize - 1] =
            yes;
    }

    /// Assuming that every element of the board until (and not including `index`) are valid, this
    /// function determines whether adding a skyscrapper of height `height` at `index` will
    /// preserve its validity.
    pub fn can_place(&self, height: u8) -> bool {
        // Check if the newly placed skyscrapper doubles without another on the same line or
        // column.
        if self.is_height_on_line(height, self.y) || self.is_height_on_column(height, self.x) {
            return false;
        }

        if !self.pushing_los[self.x].can_push(self.size as u8, height) {
            return false;
        }

        if !self.pushing_los[self.size + self.y].can_push(self.size as u8, height) {
            return false;
        }

        if !self.pulling_los[self.x].can_push(height) {
            return false;
        }

        if !self.pulling_los[self.size + self.y].can_push(height) {
            return false;
        }

        true
    }

    /// Find a valid skyscrapper to place at `index`, such that the board retains its validity.
    ///
    /// If the function backtracked to the begining of the board, the function return `false`,
    /// indicating that no solution is possible.
    pub fn place_candidate(&mut self) -> bool {
        while self.next_candidate <= self.size as u8 {
            if self.can_place(self.next_candidate) {
                // We have found a suitable height!
                let s = self.size;
                self.pushing_los[self.x].push(self.next_candidate);
                self.pushing_los[s + self.y].push(self.next_candidate);
                self.pulling_los[self.x].push(
                    &self.height_presence[s * s + self.x * s..s * s + self.x * s + s],
                    self.next_candidate,
                );
                self.pulling_los[self.size + self.y].push(
                    &self.height_presence[self.y * s..self.y * s + s],
                    self.next_candidate,
                );

                self.set_height_on_line(self.next_candidate, self.y, true);
                self.set_height_on_column(self.next_candidate, self.x, true);
                self.board[self.index] = self.next_candidate;

                // Advance the index by one position.
                self.index += 1;
                self.x = self.index % self.size;
                self.y = self.index / self.size;
                self.next_candidate = 1;

                return true;
            }

            self.next_candidate += 1;
        }

        if self.index == 0 {
            // No solution found.
            return false;
        }

        // No candidate was suitable for this position.
        // Something in the board is invalid. We have to backtrack.
        self.index -= 1;
        self.x = self.index % self.size;
        self.y = self.index / self.size;

        let old = self.board[self.index];
        self.board[self.index] = 0;
        self.set_height_on_line(old, self.y, false);
        self.set_height_on_column(old, self.x, false);
        self.next_candidate = old + 1;

        self.pushing_los[self.x].pop();
        self.pushing_los[self.size + self.y].pop();
        self.pulling_los[self.x].pop();
        self.pulling_los[self.size + self.y].pop();

        true
    }
}

/// Solves the provided header.
pub fn solve(header: &[u8], size: usize) -> Result<Box<[u8]>, SolutionError> {
    let mut board = Board::empty(header, size);

    loop {
        if crate::sigint::occured() {
            return Err(SolutionError::Interrupted);
        }

        if !board.place_candidate() {
            return Err(SolutionError::NoSolution);
        }

        if board.is_complete() {
            return Ok(board.board);
        }
    }
}

/// Solves the provided header, but animates the process.
pub fn solve_animated(
    header: &[u8],
    size: usize,
    w: &mut dyn WriteColor,
    interval: Duration,
) -> Result<Box<[u8]>, SolutionError> {
    let mut board = Board::empty(header, size);

    let _ = format::print_both(w, &board.board, header, size as u8, true);

    loop {
        if crate::sigint::occured() {
            return Err(SolutionError::Interrupted);
        }

        if !board.place_candidate() {
            return Err(SolutionError::NoSolution);
        }

        print!("\x1B[{}A\x1B[J", size + 2);
        let _ = format::print_both(w, &board.board, header, size as u8, true);
        std::thread::sleep(interval);

        if board.is_complete() {
            print!("\x1B[{}A\x1B[J", size + 2);
            return Ok(board.board);
        }
    }
}
