//! The GUI game session: owns the model state, config, move count, and an
//! undo/redo history built on the model's reversible-move API. This module has
//! no macroquad dependency, so it is unit-testable; the caller supplies elapsed
//! time (from macroquad on native/web).

use klondike::{apply_undoable, undo_move, GameConfig, GameState, Move, Undo};

pub struct Session {
    pub state: GameState,
    moves: u32,
    history: Vec<(Move, Undo)>,
    redo: Vec<Move>,
    elapsed_secs: u64,
    message: Option<String>,
    /// The deal is being (or was) completed by auto-solve, so its score and time
    /// are not counted (a future high-score feature checks `was_auto_solved`).
    auto_solving: bool,
    auto_solved: bool,
}

impl Session {
    pub fn new(seed: u64, config: GameConfig) -> Self {
        Session {
            state: GameState::new_with_seed(seed, config),
            moves: 0,
            history: Vec::new(),
            redo: Vec::new(),
            elapsed_secs: 0,
            message: None,
            auto_solving: false,
            auto_solved: false,
        }
    }

    /// Whether an auto-solve is in progress.
    pub fn is_auto_solving(&self) -> bool {
        self.auto_solving
    }

    /// Whether this game was completed by auto-solve (score/time not counted).
    pub fn was_auto_solved(&self) -> bool {
        self.auto_solved
    }

    /// Begin auto-solving: the timer freezes at zero and the finish will not be
    /// recorded as a scored win.
    pub fn begin_auto_solve(&mut self) {
        self.auto_solving = true;
    }

    /// Mark the auto-solve finished (called on reaching the won state).
    pub fn finish_auto_solve(&mut self) {
        self.auto_solving = false;
        self.auto_solved = true;
    }

    /// Cancel an in-progress auto-solve without marking it solved.
    pub fn cancel_auto_solve(&mut self) {
        self.auto_solving = false;
    }

    pub fn seed(&self) -> u64 {
        self.state.seed()
    }
    pub fn move_count(&self) -> u32 {
        self.moves
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    pub fn is_won(&self) -> bool {
        self.state.is_won()
    }
    pub fn elapsed_secs(&self) -> u64 {
        self.elapsed_secs
    }

    /// Push the elapsed play time into the model so the score is current.
    pub fn set_elapsed(&mut self, secs: u64) {
        self.elapsed_secs = secs;
        self.state.set_elapsed_secs(secs);
    }

    pub fn score(&self) -> i64 {
        self.state.current_score()
    }
    pub fn final_score(&self) -> i64 {
        self.state.final_score()
    }

    /// Set a transient status message (e.g. rejection feedback).
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    /// Apply a legal move, recording it for undo. Returns whether it applied.
    pub fn apply(&mut self, mv: Move) -> bool {
        match apply_undoable(&mut self.state, mv) {
            Ok(undo) => {
                self.history.push((mv, undo));
                self.redo.clear();
                self.moves += 1;
                self.message = None;
                true
            }
            Err(e) => {
                self.message = Some(format!("{e}"));
                false
            }
        }
    }

    /// Undo the most recent move (reverses the model exactly).
    pub fn undo(&mut self) {
        if let Some((mv, undo)) = self.history.pop() {
            undo_move(&mut self.state, mv, undo);
            self.redo.push(mv);
            self.moves = self.moves.saturating_sub(1);
            self.message = Some("Undo".to_string());
        } else {
            self.message = Some("Nothing to undo".to_string());
        }
    }

    /// Redo a previously undone move.
    pub fn redo(&mut self) {
        if let Some(mv) = self.redo.pop() {
            if let Ok(undo) = apply_undoable(&mut self.state, mv) {
                self.history.push((mv, undo));
                self.moves += 1;
                self.message = Some("Redo".to_string());
            }
        } else {
            self.message = Some("Nothing to redo".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_undo_redo_restore_state_and_counters() {
        let mut s = Session::new(42, GameConfig::default());
        assert_eq!(s.move_count(), 0);
        let before = s.state.clone();

        // A fresh deal always allows a draw.
        assert!(s.apply(Move::Draw));
        assert_eq!(s.move_count(), 1);
        assert_ne!(s.state, before);

        s.undo();
        assert_eq!(s.move_count(), 0);
        assert_eq!(s.state, before, "undo restores the exact prior state");

        s.redo();
        assert_eq!(s.move_count(), 1);
        assert!(s.state.waste.top().is_some());
    }

    #[test]
    fn new_game_resets() {
        let mut s = Session::new(1, GameConfig::default());
        s.apply(Move::Draw);
        s.apply(Move::Draw);
        assert_eq!(s.move_count(), 2);
        let fresh = Session::new(2, GameConfig::default());
        assert_eq!(fresh.move_count(), 0);
        assert!(fresh.message().is_none());
    }

    #[test]
    fn a_new_move_clears_redo() {
        let mut s = Session::new(7, GameConfig::default());
        s.apply(Move::Draw);
        s.undo(); // redo now has one
        s.apply(Move::Draw); // a new move must clear redo
        s.redo(); // nothing to redo
        assert_eq!(s.message(), Some("Nothing to redo"));
    }
}
