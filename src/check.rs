//! Provides ways to check whether a given board is valid.

/// A kind of [`BoardError`].
pub enum BoardErrorKind {
    /// The number is invalid.
    InvalidNumber,
    /// There is not enough columns.
    ColumnCount { expected: u8, given: u8 },
    /// There is not enough rows.
    RowCount { expected: u8, given: u8 },
    /// Invalid character found in the input.
    UnexpectedCharacter(u8),
    /// Invalid view count from top to bottom.
    TopToBottom { expected: u8, given: u8 },
    /// Invalid view count from bottom to top.
    BottomToTop { expected: u8, given: u8 },
    /// Invalid view count from left to right.
    LeftToRight { expected: u8, given: u8 },
    /// Invalid view count from right to left.
    RightToLeft { expected: u8, given: u8 },
    /// Doubles found.
    Doubles,
}

/// An error which might occur when checking a board.
pub struct BoardError {
    /// The kind of the error.
    pub kind: BoardErrorKind,
    /// The spans of this error.
    pub spans: Vec<Span>,
}

/// A span.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// A parsed board cell.
pub struct BoardCell {
    /// The value of the cell.
    value: u8,
    span: Span,
}

fn parse(number: &[u8]) -> Option<u8> {
    let mut result = 0u8;

    for &b in number {
        let value = b.wrapping_sub(b'0');
        if value > 9 {
            return None;
        }
        result = result.checked_mul(10)?.checked_add(value)?;
    }

    Some(result)
}

/// Parses the provided ASCII board into an actual board.
fn parse_board(board: &[u8], size: u8) -> Result<Box<[BoardCell]>, BoardError> {
    let mut result = Vec::new();

    let mut in_number = false;
    let mut l_start = 0;
    let mut n_start = 0;
    let mut numbers_on_line = 0;
    let mut lines = 0;
    let mut i = 0;
    while i < board.len() {
        if !in_number {
            match board[i] {
                b' ' => i += 1,
                b'\n' => {
                    if numbers_on_line != size {
                        return Err(BoardError {
                            kind: BoardErrorKind::ColumnCount {
                                expected: size,
                                given: numbers_on_line,
                            },
                            spans: vec![Span {
                                start: l_start,
                                end: i,
                            }],
                        });
                    }

                    i += 1;
                    numbers_on_line = 0;
                    l_start = i;
                    lines += 1;
                }
                b'0'..=b'9' => {
                    n_start = i;
                    in_number = true;
                }
                c => {
                    return Err(BoardError {
                        kind: BoardErrorKind::UnexpectedCharacter(c),
                        spans: vec![Span {
                            start: i,
                            end: i + 1,
                        }],
                    });
                }
            }
        } else {
            match board[i] {
                b'0'..=b'9' => i += 1,
                _ => match parse(&board[n_start..i]) {
                    Some(value) => {
                        if value > size || value == 0 {
                            return Err(BoardError {
                                kind: BoardErrorKind::InvalidNumber,
                                spans: vec![Span {
                                    start: n_start,
                                    end: i,
                                }],
                            });
                        }
                        numbers_on_line += 1;
                        result.push(BoardCell {
                            value,
                            span: Span {
                                start: n_start,
                                end: i,
                            },
                        });
                        in_number = false;
                    }
                    None => {
                        return Err(BoardError {
                            kind: BoardErrorKind::InvalidNumber,
                            spans: vec![Span {
                                start: n_start,
                                end: i,
                            }],
                        });
                    }
                },
            }
        }
    }

    if lines == 0 {
        return Err(BoardError {
            kind: BoardErrorKind::RowCount {
                expected: size,
                given: lines,
            },
            spans: vec![Span { start: 0, end: 0 }],
        });
    }

    if numbers_on_line != 0 {
        lines += 1;
    }

    if lines != size {
        return Err(BoardError {
            kind: BoardErrorKind::RowCount {
                expected: size,
                given: lines,
            },
            spans: vec![Span { start: 0, end: 0 }],
        });
    }

    Ok(result.into_boxed_slice())
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

/// Checks whether `board` is valid.
///
/// `board` is the ASCII representation of the board.
pub fn check(header: &[u8], size: usize, board: &[u8]) -> Result<(), BoardError> {
    let board = parse_board(board, size as u8)?;

    for k in 0..size {
        for i in 0..size {
            for j in i + 1..size {
                if board[k * size + i].value == board[k * size + j].value {
                    return Err(BoardError {
                        kind: BoardErrorKind::Doubles,
                        spans: vec![board[k * size + i].span, board[k * size + j].span],
                    });
                }

                if board[i * size + k].value == board[j * size + k].value {
                    return Err(BoardError {
                        kind: BoardErrorKind::Doubles,
                        spans: vec![board[i * size + k].span, board[j * size + k].span],
                    });
                }
            }
        }
    }

    for i in 0..size {
        // top-to-bottom
        let from_top = count_viewed(size as u8, &mut |y| board[i + y * size].value);
        if from_top != header[i] {
            return Err(BoardError {
                kind: BoardErrorKind::TopToBottom {
                    expected: header[i],
                    given: from_top,
                },
                spans: (0..size).map(|y| board[i + y * size].span).collect(),
            });
        }

        // bottom-to-top
        let from_bottom = count_viewed(size as u8, &mut |y| board[i + (size - y - 1) * size].value);
        if from_bottom != header[size + i] {
            return Err(BoardError {
                kind: BoardErrorKind::BottomToTop {
                    expected: header[size + i],
                    given: from_bottom,
                },
                spans: (0..size).map(|y| board[i + y * size].span).collect(),
            });
        }

        // left-to-right
        let from_left = count_viewed(size as u8, &mut |x| board[x + i * size].value);
        if from_left != header[size * 2 + i] {
            return Err(BoardError {
                kind: BoardErrorKind::LeftToRight {
                    expected: header[size * 2 + i],
                    given: from_left,
                },
                spans: vec![board[i * size].span, board[i * size + size - 1].span],
            });
        }

        // right-to-left
        let from_right = count_viewed(size as u8, &mut |x| board[(size - x - 1) + i * size].value);
        if from_right != header[size * 3 + i] {
            return Err(BoardError {
                kind: BoardErrorKind::RightToLeft {
                    expected: header[size * 3 + i],
                    given: from_right,
                },
                spans: vec![board[i * size].span, board[i * size + size - 1].span],
            });
        }
    }

    Ok(())
}
