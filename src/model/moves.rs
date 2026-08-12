//! The player-action vocabulary and the error type for illegal moves.
//!
//! Foundations are indexed `0..4` and tableau columns `0..7`.

use std::fmt;

/// A single player action in classic Klondike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// Draw from the stock to the waste (one or three cards per the draw mode).
    Draw,
    /// Recycle the waste back into the stock (only when the stock is empty).
    Recycle,
    /// Move the top of the waste onto foundation `foundation`.
    WasteToFoundation { foundation: usize },
    /// Move the top of the waste onto tableau column `column`.
    WasteToTableau { column: usize },
    /// Move a face-up run of `count` cards from tableau column `from` to `to`.
    /// `count == 1` moves a single card.
    TableauToTableau {
        from: usize,
        to: usize,
        count: usize,
    },
    /// Move the top of tableau column `column` onto foundation `foundation`.
    TableauToFoundation { column: usize, foundation: usize },
    /// Move the top of foundation `foundation` back onto tableau column `column`.
    FoundationToTableau { foundation: usize, column: usize },
}

/// Why a move could not be applied. When `apply_move` returns an error the game
/// state is guaranteed unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// A foundation or column index was out of range.
    IndexOutOfRange,
    /// The source pile had no card (or run) to move.
    EmptySource,
    /// The stock was empty, so it cannot be drawn from.
    StockEmpty,
    /// A recycle was attempted while the stock still had cards.
    StockNotEmpty,
    /// The configured redeal limit has been reached; no more recycles allowed.
    RedealLimitReached,
    /// The requested run length is not a valid face-up run in the source column.
    InvalidRun,
    /// The card does not satisfy the tableau placement rule.
    IllegalTableauPlacement,
    /// The card does not satisfy the foundation placement rule.
    IllegalFoundationPlacement,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MoveError::IndexOutOfRange => "pile index out of range",
            MoveError::EmptySource => "source pile is empty",
            MoveError::StockEmpty => "stock is empty",
            MoveError::StockNotEmpty => "stock is not empty; cannot recycle",
            MoveError::RedealLimitReached => "redeal limit reached",
            MoveError::InvalidRun => "invalid tableau run length",
            MoveError::IllegalTableauPlacement => "illegal tableau placement",
            MoveError::IllegalFoundationPlacement => "illegal foundation placement",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for MoveError {}
