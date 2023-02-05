//! Provides ways to solve skyscrapper problems.

use std::time::Duration;

use termcolor::WriteColor;

/// An error which may occur whilst trying to compute a solution.
pub enum SolutionError {
    /// No solution was found for the provided header.
    NoSolution,
    /// The alogithm has been interrupted.
    Interrupted,
}

/// No solution is possible.
struct NoSolution;

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
    /// Note that this function does not check whether the cell actually allows this value.
    pub fn set(&mut self, value: u8) {
        debug_assert!(self.accepts(value));

        // SAFETY:
        //  `BoardCell` knows that its length is greater or equal to `2`.
        unsafe {
            *self.0.get_unchecked_mut(0) = 1;
            *self.0.get_unchecked_mut(1) = value;
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

    /// Returns an iterator over the cells of this board.
    pub fn cell_indices(&self) -> impl '_ + Clone + Iterator<Item = (usize, &BoardCell)> {
        (0..self.size * self.size).map(move |i| {
            let index = (self.size + 1) * i;
            (index, unsafe { self.cell(index) })
        })
    }

    fn _remove_duplicates(
        &mut self,
        x: usize,
        y: usize,
        value: u8,
        now_fixed: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
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

        for row in 0..self.size {
            // Don't try to remove duplicates on the value that we just set.
            if row == x {
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

        cell.set(value);

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
}

/// A board that remembers where it stopped backtracking.
///
/// This type is used to backtrack as far as possible witout having to clone the board.
struct BacktrackingBoard {
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

impl BacktrackingBoard {
    /// Tries to continue backtracking using the saved state.
    ///
    /// In case something goes wront (no solution possible), the state of the board is restored
    /// using `restore` and `NoSolution` is returned.
    ///
    /// Otherwise, `Ok(())` is returned and the modified state is conserved.
    ///
    /// `buf` will be cleared and used during the algorithm.
    pub fn try_backtrack(
        &mut self,
        restore: &BoardSet,
        buf: &mut Vec<(usize, usize)>,
    ) -> Result<(), NoSolution> {
        buf.clear();

        let x = self.current_index % self.set.size;
        let y = self.current_index / self.set.size;

        match unsafe {
            self.set
                .set_and_remove_duplicates(x, y, self.current_subindex, buf)
        } {
            Ok(()) => (),
            Err(err) => {
                self.set.array.copy_from_slice(&restore.array);
                return Err(err);
            }
        }

        while let Some((x, y)) = buf.pop() {
            match unsafe { self.set.remove_duplicates_around(x, y, buf) } {
                Ok(()) => (),
                Err(err) => {
                    self.set.array.copy_from_slice(&restore.array);
                    return Err(err);
                }
            }
        }

        Ok(())
    }
}

/// Solves the provided header.
pub fn solve(header: &[u8], size: usize) -> Result<Box<[u8]>, SolutionError> {
    let _ = (header, size);
    todo!();
}

/// Solves the provided header, but animates the process.
pub fn solve_animated(
    header: &[u8],
    size: usize,
    w: &mut dyn WriteColor,
    interval: Duration,
) -> Result<Box<[u8]>, SolutionError> {
    let _ = (header, size, w, interval);
    todo!();
}
