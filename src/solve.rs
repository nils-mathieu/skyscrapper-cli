//! Provides ways to solve skyscrapper problems.

use std::time::Duration;

use termcolor::WriteColor;

use crate::sigint;

/// An error which may occur whilst trying to compute a solution.
pub enum SolutionError {
    /// No solution was found for the provided header.
    NoSolution,
    /// The alogithm has been interrupted.
    Interrupted,
}

/// No solution is possible.
struct NoSolution;

impl From<NoSolution> for SolutionError {
    fn from(_value: NoSolution) -> Self {
        Self::NoSolution
    }
}

/// Contains the values available for a given board cell.
#[repr(transparent)]
struct BoardCell([u8]);

impl BoardCell {
    /// Creates a new [`BoardCell`] instance.
    ///
    /// # Safety
    ///
    /// * This function assumes `slice` has a length of at least `2`.
    /// * And that its first element is smaller than its length.
    unsafe fn wrap_ref(slice: &[u8]) -> &Self {
        debug_assert!(slice.len() >= 2);
        debug_assert!((slice[0] as usize) < slice.len());

        // SAFETY:
        //  - `BoardCell` is a `#[repr(transparent)]` wrapper around `[u8]`.
        unsafe { core::mem::transmute(slice) }
    }

    /// Creates a new [`BoardCell`] instance.
    ///
    /// # Safety
    ///
    /// * This function assumes `slice` has a length of at least `2`.
    /// * And that its first element is smaller than its length.
    unsafe fn wrap_mut(slice: &mut [u8]) -> &mut Self {
        debug_assert!(slice.len() >= 2);
        debug_assert!((slice[0] as usize) < slice.len());

        // SAFETY:
        //  - `BoardCell` is a `#[repr(transparent)]` wrapper around `[u8]`.
        unsafe { core::mem::transmute(slice) }
    }

    /// Returns whether this cell accepts a certain value.
    pub fn accepts(&self, value: u8) -> bool {
        self.0.contains(&value)
    }

    /// Returns the number of element allowed for this cell.
    pub fn count(&self) -> usize {
        // SAFETY:
        //  `BoardCell` knows that its slice has a length greater or equal to `2`.
        unsafe { *self.0.get_unchecked(0) as usize }
    }

    /// Returns a slice over the values allowed by this cell.
    pub fn slice(&self) -> &[u8] {
        let len = self.count();

        // SAFETY:
        //  The slice is known to be large enough to store `len + 1` elements.
        unsafe { self.0.get_unchecked(1..1 + len) }
    }

    /// Sets the value of this cell to `value`.
    ///
    /// If the cell forbids the provided value, an error is returned.
    pub fn set(&mut self, value: u8) -> Result<(), NoSolution> {
        if self.accepts(value) {
            // SAFETY:
            //  `BoardCell` knows that its length is greater or equal to `2`.
            unsafe {
                *self.0.get_unchecked_mut(0) = 1;
                *self.0.get_unchecked_mut(1) = value;
            }

            Ok(())
        } else {
            Err(NoSolution)
        }
    }

    /// Tries to disallow a value for this cell.
    ///
    /// If the value was already disallowed, `false` is returned. Otherwise, `true` is returned.
    pub fn forbid(&mut self, value: u8) -> bool {
        if let Some(pos) = self.slice().iter().position(|&b| b == value) {
            unsafe {
                // SAFETY:
                //  The size of the inner slice is known to be larger than `2`.
                *self.0.get_unchecked_mut(0) -= 1;
                let len = *self.0.get_unchecked(0) as usize;

                // SAFETY:
                //  `pos` has been returned
                *self.0.get_unchecked_mut(1 + pos) = *self.0.get_unchecked(1 + len);
            }

            true
        } else {
            false
        }
    }
}

/// Stores every possible value available for each cell of a board.
#[derive(Clone)]
struct BoardSet {
    /// The backing array of this [`BoardSet`].
    ///
    /// This array has a size of `size * size * (size + 1)` bytes. Where `size` is the size of the
    /// input skyscrapper board.
    ///
    /// Each cell takes `size + 1` bytes. The first byte represents how many possible values the
    /// cell has, and the `size` other bytes are the actual possible values.
    ///
    /// For example:
    ///
    /// ```txt
    /// |3|1|2|5|4|3|
    /// ```
    ///
    /// When a size of `5`, above cell accepts the values 1, 2, and 5. 3 values in total.
    array: Box<[u8]>,
    /// The size that was used to create the `BoardSet`.
    ///
    /// This tiny bit of redundancy makes the program much more safe and easy to use and maintain.
    size: usize,
}

impl BoardSet {
    /// Creates a new [`BoardSet`] instance.
    ///
    /// Every cell of the created board will accept every possible value.
    pub fn new(size: usize) -> Self {
        let mut array = Vec::with_capacity(size * size * (size + 1));

        for _ in 0..size * size {
            array.push(size as u8);
            array.extend(1..=size as u8);
        }

        debug_assert_eq!(array.len(), array.capacity());
        debug_assert_eq!(array.capacity(), size * size * (size + 1));

        Self {
            array: array.into_boxed_slice(),
            size,
        }
    }

    /// Gets the state of a cell of this [`Board`].
    ///
    /// # Safety
    ///
    /// `index` must be the start of a cell bounary.
    /// `index` must be on a cell bounary.
    pub unsafe fn cell_mut(&mut self, index: usize) -> &mut BoardCell {
        debug_assert_eq!(index % (self.size + 1), 0);
        debug_assert!(index < self.array.len());

        // SAFETY:
        //  `index` is on a cell bounary, ensuring that this slice is in bounds.
        let slice = unsafe { self.array.get_unchecked_mut(index..index + 1 + self.size) };

        // SAFETY:
        //  We know by invariant of `BoardSet` that each cell contains `size + 1` elements, ensuring
        //  that both preconditions are validated.
        unsafe { BoardCell::wrap_mut(slice) }
    }

    /// Gets the state of a cell of this [`Board`].
    ///
    /// # Safety
    ///
    /// `index` must be the start of a cell bounary.
    pub unsafe fn cell(&self, index: usize) -> &BoardCell {
        debug_assert_eq!(index % (self.size + 1), 0);
        debug_assert!(index < self.array.len());

        // SAFETY:
        //  `index` is on a cell bounary, ensuring that this slice is in bounds.
        let slice = unsafe { self.array.get_unchecked(index..index + 1 + self.size) };

        // SAFETY:
        //  We know by invariant of `BoardSet` that each cell contains `size + 1` elements, ensuring
        //  that both preconditions are validated.
        unsafe { BoardCell::wrap_ref(slice) }
    }

    /// Account for a specific header value associated with a collection of indices.
    ///
    /// Cells that are set to a single value are added to `buf`.
    ///
    /// # Safety
    ///
    /// `indices` must return valid cell coordinates.
    unsafe fn _account_for_header(
        &mut self,
        value: u8,
        mut indices: impl Iterator<Item = (usize, usize)>,
        buf: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        let size = self.size as u8;

        if value == 1 {
            // The value one only allows for the maximum value directly before itself.
            let (x, y) = indices.next().unwrap();
            let index = x * (self.size + 1) + y * (self.size + 1) * self.size;
            // SAFETY:
            //  The iterator must provide valid cell indices.
            let cell = unsafe { self.cell_mut(index) };
            cell.set(size)?;
            buf.push((x, y));
            return Ok(());
        } else if value == self.size as u8 {
            // The maximum value only allows one configuration.
            for (i, (x, y)) in indices.enumerate() {
                let index = x * (self.size + 1) + y * (self.size + 1) * self.size;
                // SAFETY:
                //  The iterator must provide valid indices.
                let cell = unsafe { self.cell_mut(index) };

                cell.set((i + 1) as u8)?;
                buf.push((x, y));
            }
            return Ok(());
        }

        for (i, (x, y)) in indices.enumerate() {
            let index = x * (self.size + 1) + y * self.size * (self.size + 1);

            // SAFETY:
            //  `indices` must yield valid cell indices.
            let cell = unsafe { self.cell_mut(index) };

            let first_to_remove = size - value + 2 + i as u8;
            for to_remove in first_to_remove..=size {
                if cell.forbid(to_remove) {
                    match cell.count() {
                        0 => return Err(NoSolution),
                        1 => buf.push((x, y)),
                        _ => (),
                    }
                }
            }
        }

        Ok(())
    }

    // TODO:
    //  This function seems to add the same coordinates multiple times (up to four times in the
    //  worst case) to the buffer. Being able to mitigate that would be great.
    //
    /// Modifies the allowed values for each cell of this board using the provided header-line.
    pub fn account_for_header(
        &mut self,
        header: &[u8],
        buf: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        let size = self.size;

        assert_eq!(header.len(), size * 4);

        for col in 0..size {
            // SAFETY:
            //  We know that the header has a size of `size * 4`.
            let views_from_top = unsafe { *header.get_unchecked(col) };

            unsafe {
                self._account_for_header(views_from_top, (0..size).map(|y| (col, y)), buf)?;
            }

            let views_from_bottom = unsafe { *header.get_unchecked(size + col) };

            unsafe {
                self._account_for_header(views_from_bottom, (0..size).rev().map(|y| (col, y)), buf)
            }?;
        }

        for row in 0..size {
            let views_from_left = unsafe { *header.get_unchecked(size * 2 + row) };

            unsafe {
                self._account_for_header(views_from_left, (0..size).map(|x| (x, row)), buf)?;
            }

            let views_from_right = unsafe { *header.get_unchecked(size * 3 + row) };

            unsafe {
                self._account_for_header(views_from_right, (0..size).rev().map(|x| (x, row)), buf)?;
            }
        }

        Ok(())
    }

    fn _remove_duplicates(
        &mut self,
        x: usize,
        y: usize,
        value: u8,
        now_fixed: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        // same line
        for col in 0..self.size {
            // Don't try to remove duplicates on the value that we just set.
            if col == x {
                continue;
            }

            let index = (self.size + 1) * col + (self.size + 1) * self.size * y;
            let cell = unsafe { self.cell_mut(index) };
            if cell.forbid(value) {
                match cell.count() {
                    0 => return Err(NoSolution),
                    1 => now_fixed.push((col, y)),
                    _ => (),
                }
            };
        }

        // same column
        for row in 0..self.size {
            // Don't try to remove duplicates on the value that we just set.
            if row == y {
                continue;
            }

            let index = (self.size + 1) * x + (self.size + 1) * self.size * row;
            let cell = unsafe { self.cell_mut(index) };
            if cell.forbid(value) {
                match cell.count() {
                    0 => return Err(NoSolution),
                    1 => now_fixed.push((x, row)),
                    _ => (),
                }
            };
        }

        Ok(())
    }

    /// Sets the provided cell to `value` and forbids duplicates around that value.
    ///
    /// # Safety
    ///
    /// * `x` and `y` must be in bounds.
    /// * `subindex` must be in bounds.
    pub unsafe fn set_and_remove_duplicates(
        &mut self,
        x: usize,
        y: usize,
        subindex: usize,
        now_fixed: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        debug_assert!(x < self.size);
        debug_assert!(y < self.size);

        let index = (self.size + 1) * x + (self.size + 1) * self.size * y;

        // SAFETY:
        //  If `x` and `y` are in bounds, then `index` is a valid index.
        let cell = unsafe { self.cell_mut(index) };

        // SAFETY:
        //  The caller must provide a valid subindex.
        let value = unsafe { *cell.slice().get_unchecked(subindex) };

        cell.set(value)?;

        self._remove_duplicates(x, y, value, now_fixed)
    }

    /// Assumes that the cell `(x, y)` allows one value and forbids any duplicate in cells on the
    /// same colum or row.
    ///
    /// If removing duplicates for a cell disallows *every* value, `NoSolution` is returned.
    ///
    /// The cells that now contain only one possible value are pushed to `now_fixed`.
    ///
    /// # Safety
    ///
    /// * `x` and `y` must be in bounds (less than the size).
    /// * The cell at that position must allow exactly one value.
    pub unsafe fn remove_duplicates_around(
        &mut self,
        x: usize,
        y: usize,
        now_fixed: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        debug_assert!(x < self.size);
        debug_assert!(y < self.size);

        let index = (self.size + 1) * x + (self.size + 1) * self.size * y;

        // SAFETY:
        //  If `x` and `y` are in bounds, then `index` is a valid index.
        let cell = unsafe { self.cell(index) };

        debug_assert_eq!(cell.count(), 1);

        // SAFETY:
        //  The caller must make sure that this cell contains at least one value.
        let value = unsafe { *cell.slice().get_unchecked(0) };

        self._remove_duplicates(x, y, value, now_fixed)
    }

    /// Removes the duplicates around the values specified in the provided vector, leaving that
    /// vector empty.
    pub fn remove_duplicates_in(
        &mut self,
        buf: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        while let Some((x, y)) = buf.pop() {
            unsafe { self.remove_duplicates_around(x, y, buf)? };
        }

        Ok(())
    }

    /// Assumes that the board is complete and turns it into a normal board.
    pub fn create_board(&self) -> Box<[u8]> {
        (0..self.size * self.size)
            .map(|i| {
                let index = i * (self.size + 1);
                let cell = unsafe { self.cell(index) };
                if cell.count() == 1 {
                    cell.slice()[0]
                } else {
                    0
                }
            })
            .collect()
    }
}

/// A board that remembers where it stopped backtracking.
///
/// This type is used to backtrack as far as possible witout having to clone the board.
struct BacktrackingBoard {
    /// The original [`BoardSet`], used when actually backtracking.
    original: BoardSet,
    /// The inner [`BoardSet`] instance.
    set: BoardSet,
    /// The index of the cell on which we are currently backtracking.
    ///
    /// This is always less than `size * size`.
    ///
    /// Every cell *before* that index are fixed to a single value.
    current_index: usize,
    /// The index of the value that we will choose next to backtrack.
    ///
    /// This is always in bound of the cell's possibilities.
    current_subindex: usize,
}

/// An error which may occur when backtracking.
#[derive(Debug)]
enum BacktrackError {
    /// There is no solution.
    NoSolution,
    /// No solition was found during *this* try, but it's still possible try again.
    Retry,
}

impl BacktrackingBoard {
    /// Creates a new [`BacktrackingBoard`] from the provided [`BoardSet`].
    ///
    /// If the provided board is already complete, the function returns [`Err`] with the input
    /// [`BoardSet`].
    pub fn new(set: BoardSet) -> Result<Self, BoardSet> {
        let mut current_index = 0;

        while current_index < set.size * set.size
            && unsafe { set.cell(current_index * (set.size + 1)) }.count() == 1
        {
            current_index += 1;
        }

        if current_index == set.size * set.size {
            return Err(set);
        }

        Ok(Self {
            original: set.clone(),
            set,
            current_index,
            current_subindex: 0,
        })
    }

    fn _try_backtrack(&mut self, buf: &mut Vec<(usize, usize)>) -> Result<(), NoSolution> {
        buf.clear();

        let x = self.current_index % self.set.size;
        let y = self.current_index / self.set.size;

        unsafe {
            self.set
                .set_and_remove_duplicates(x, y, self.current_subindex, buf)?
        };

        self.set.remove_duplicates_in(buf)
    }

    // TODO: possible optimization
    //  If we store the total number of "one" cells, we can check easily whether the board is
    //  complete or not, AND we can start backtracking on the cells that are the most efficient
    //  with the least amount of possibilities. We might even be able to cache this too to save
    //  the lookup.
    //
    //  At the moment, we are backtracking from top-left to bottom-right and we know that we're done
    //  when the backtracking index reaches the end; meaning that `remove_duplicates_around` is not
    //  as optimized as it could be. In this state, we could simply check for duplicates *after*
    //  the input index.
    //
    //  Something else: we store the "original" board in the `BacktrackingBoard`. Meaning that the
    //  final stack of `BacktrackingBoard` instance will duplicate one board each.
    //
    //  It's probably possible to multi-thread this. Each "fork" is independ from the others, and
    //  we could spawn a new task for every possible subindex.
    //
    /// Tries to continue backtracking using the current state. When an error occurs (no solution is
    /// possible from this state), the internal state is restored.
    ///
    /// Calling this function again in case of error always produces an error.
    ///
    /// Otherwise, `Ok(())` is returned and the modified state is conserved.
    ///
    /// `buf` will be cleared and used during the algorithm.
    pub fn try_backtrack(&mut self, buf: &mut Vec<(usize, usize)>) -> Result<(), BacktrackError> {
        self.set.array.copy_from_slice(&self.original.array);

        let count = unsafe { self.set.cell(self.current_index * (self.set.size + 1)) }.count();
        if self.current_subindex == count {
            // We are out of possible values. There is no possible solution.
            return Err(BacktrackError::NoSolution);
        }

        let result = self._try_backtrack(buf);
        self.current_subindex += 1;
        match result {
            Ok(()) => Ok(()),
            Err(_) => Err(BacktrackError::Retry),
        }
    }
}

/// Solves the provided header.
pub fn solve(header: &[u8], size: usize) -> Result<Box<[u8]>, SolutionError> {
    let mut buf = Vec::new();
    let mut set = BoardSet::new(size);
    set.account_for_header(header, &mut buf)?;
    set.remove_duplicates_in(&mut buf)?;

    let mut backtrackers = Vec::new();

    match BacktrackingBoard::new(set) {
        Ok(ok) => backtrackers.push(ok),
        Err(complete) => return Ok(complete.create_board()),
    };

    loop {
        if sigint::occured() {
            return Err(SolutionError::Interrupted);
        }

        let backtracker = backtrackers.last_mut().unwrap();
        match backtracker.try_backtrack(&mut buf) {
            // TODO:
            //  calling `new` here re-computes `current_index` from the start. We should create a
            //  special `new_backtracking_fork` function that keeps the index (or something like
            //  that).
            Ok(()) => match BacktrackingBoard::new(backtracker.set.clone()) {
                Ok(ok) => backtrackers.push(ok),
                Err(complete) => return Ok(complete.create_board()),
            },
            Err(BacktrackError::NoSolution) => {
                backtrackers.pop();
                if backtrackers.is_empty() {
                    return Err(SolutionError::NoSolution);
                }
            }
            Err(BacktrackError::Retry) => (),
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
    let mut buf = Vec::new();
    let mut set = BoardSet::new(size);
    set.account_for_header(header, &mut buf)?;
    set.remove_duplicates_in(&mut buf)?;

    let _ = crate::format::print_solution(
        w,
        &set.create_board(),
        header,
        size as u8,
        &crate::args::OutputFormat::Both,
    );

    let mut backtrackers = Vec::new();

    match BacktrackingBoard::new(set) {
        Ok(ok) => backtrackers.push(ok),
        Err(complete) => return Ok(complete.create_board()),
    };

    loop {
        if sigint::occured() {
            return Err(SolutionError::Interrupted);
        }

        let backtracker = backtrackers.last_mut().unwrap();

        print!("\x1B[{}A\x1B[J", size + 2);
        let _ = crate::format::print_solution(
            w,
            &backtracker.set.create_board(),
            header,
            size as u8,
            &crate::args::OutputFormat::Both,
        );
        std::thread::sleep(interval);

        match backtracker.try_backtrack(&mut buf) {
            // TODO:
            //  calling `new` here re-computes `current_index` from the start. We should create a
            //  special `new_backtracking_fork` function that keeps the index (or something like
            //  that).
            Ok(()) => match BacktrackingBoard::new(backtracker.set.clone()) {
                Ok(ok) => backtrackers.push(ok),
                Err(complete) => {
                    print!("\x1B[{}A\x1B[J", size + 2);
                    return Ok(complete.create_board());
                }
            },
            Err(BacktrackError::NoSolution) => {
                backtrackers.pop();
                if backtrackers.is_empty() {
                    return Err(SolutionError::NoSolution);
                }
            }
            Err(BacktrackError::Retry) => (),
        }
    }
}
