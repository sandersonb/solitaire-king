//! The interactive terminal CLI for playing Klondike.
//!
//! Layered so the game logic is testable without a terminal:
//! - [`input`]  — key → pile mapping and pure move-resolution helpers.
//! - [`session`] — the game session: state, undo/redo, history, dispatch.
//! - [`render`] — drawing the board with color and Unicode (the only I/O layer).

pub mod input;
pub mod render;
pub mod session;
pub mod solve;

/// A pile the player can address with a key. Tableau columns are `0..7`,
/// foundations `0..4`; `StockWaste` is the stock/waste corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pile {
    Tableau(usize),
    Foundation(usize),
    StockWaste,
}

/// What the main loop should do after handling a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Keep looping.
    Continue,
    /// Exit the game.
    Quit,
}
