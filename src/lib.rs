//! Core model for Klondike Solitaire.
//!
//! This crate is a pure, deterministic, I/O-free library implementing the
//! classic ("standard") rules of Klondike Solitaire together with Microsoft
//! Windows Solitaire "Standard" scoring. It is the shared foundation for the
//! CLI, interactive UI, and automatic solver that will be built on top of it.
//!
//! # Terminology
//!
//! The vocabulary follows standard Klondike terminology:
//! - **Stock**: the face-down draw pile.
//! - **Waste** (talon): the face-up pile cards are drawn onto.
//! - **Foundation**: the four piles built up by suit from Ace to King.
//! - **Tableau**: the seven columns built down in alternating colors.
//!
//! # Determinism
//!
//! A game is created from a `u64` seed. The same `(seed, GameConfig)` always
//! produces the identical deal, using an in-crate PRNG (see [`model::rng`]) so
//! deals are reproducible and portable across platforms.

pub mod model;
pub mod solver;

pub use model::card::{Card, Color, Rank, Suit};
pub use model::deck::Deck;
pub use model::moves::{Move, MoveError};
pub use model::pile::{Foundation, Stock, TableauColumn, Waste};
pub use model::rules::{apply_undoable, legal_moves, undo_move, Undo};
pub use model::score::{
    Score, ScoreEvent, SCORE_FLIP_TABLEAU, SCORE_FOUNDATION_TO_TABLEAU,
    SCORE_TABLEAU_TO_FOUNDATION, SCORE_WASTE_TO_FOUNDATION, SCORE_WASTE_TO_TABLEAU,
};
pub use model::state::{DrawMode, GameConfig, GameState};
pub use solver::{
    solve, solve_state, validate_empty_column_symmetry, validate_equivalence,
    validate_key_strategy, validate_move_ordering, validate_no_op, validate_safe_automoves,
    validate_transposition_table, zobrist, KeyStrategy, SolveBudget, SolveOptions, SolveResult,
    ValidationOutcome, Verdict, DEFAULT_MAX_TABLE_ENTRIES,
};
