//! Game configuration and the `GameState` aggregate.

use crate::model::deal;
use crate::model::moves::{Move, MoveError};
use crate::model::pile::{Foundation, Stock, TableauColumn, Waste};
use crate::model::rules;
use crate::model::score::Score;

/// How many cards are turned from the stock per draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    /// Draw one card at a time.
    One,
    /// Draw three cards at a time (classic Windows default).
    Three,
}

impl DrawMode {
    /// The number of cards this mode turns per draw.
    pub fn count(self) -> usize {
        match self {
            DrawMode::One => 1,
            DrawMode::Three => 3,
        }
    }
}

/// Configuration selected for a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    /// Draw one or three cards per stock draw.
    pub draw_mode: DrawMode,
    /// Maximum number of stock recycles allowed. `None` means unlimited.
    pub redeal_limit: Option<u32>,
    /// Whether timed scoring (time penalty + win bonus) is in effect.
    pub timed: bool,
}

impl Default for GameConfig {
    /// Classic Windows defaults: draw-three, unlimited recycles, untimed.
    fn default() -> Self {
        GameConfig {
            draw_mode: DrawMode::Three,
            redeal_limit: None,
            timed: false,
        }
    }
}

/// The complete state of a game in progress — the single source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub stock: Stock,
    pub waste: Waste,
    pub foundations: [Foundation; 4],
    pub tableau: [TableauColumn; 7],
    seed: u64,
    config: GameConfig,
    score: Score,
    /// Elapsed play time in seconds, supplied by the caller (the front-end owns
    /// the clock). Only affects the score in timed mode.
    elapsed_secs: u64,
    /// How many times the waste has been recycled into the stock so far.
    recycles_done: u32,
}

impl GameState {
    /// Create and deal a new game from `seed` and `config`. The same
    /// `(seed, config)` always produces the identical initial layout.
    pub fn new_with_seed(seed: u64, config: GameConfig) -> Self {
        let dealt = deal::deal(seed);
        GameState {
            stock: dealt.stock,
            waste: dealt.waste,
            foundations: dealt.foundations,
            tableau: dealt.tableau,
            seed,
            config,
            score: Score::new(),
            elapsed_secs: 0,
            recycles_done: 0,
        }
    }

    /// The seed this game was dealt from.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The configuration in effect for this game.
    pub fn config(&self) -> GameConfig {
        self.config
    }

    /// The running score object.
    pub fn score(&self) -> Score {
        self.score
    }

    /// Mutable access to the running score (used by the rules engine).
    pub(crate) fn score_mut(&mut self) -> &mut Score {
        &mut self.score
    }

    /// Number of recycles performed so far.
    pub fn recycles_done(&self) -> u32 {
        self.recycles_done
    }

    /// Record a recycle (used by the rules engine).
    pub(crate) fn record_recycle(&mut self) {
        self.recycles_done += 1;
    }

    /// Restore the score and recycle count (used by `undo_move`).
    pub(crate) fn restore_score_and_recycles(&mut self, score: Score, recycles: u32) {
        self.score = score;
        self.recycles_done = recycles;
    }

    /// Whether a recycle is currently permitted by the redeal limit.
    pub fn recycle_allowed(&self) -> bool {
        match self.config.redeal_limit {
            None => true,
            Some(limit) => self.recycles_done < limit,
        }
    }

    /// Set the elapsed play time in seconds (the caller owns the clock).
    pub fn set_elapsed_secs(&mut self, secs: u64) {
        self.elapsed_secs = secs;
    }

    /// The elapsed play time in seconds.
    pub fn elapsed_secs(&self) -> u64 {
        self.elapsed_secs
    }

    /// The current displayable score, accounting for timed play.
    pub fn current_score(&self) -> i64 {
        self.score.current(self.elapsed_secs, self.config.timed)
    }

    /// The final score, including the timed win bonus if the game is won.
    pub fn final_score(&self) -> i64 {
        self.score
            .final_score(self.elapsed_secs, self.config.timed, self.is_won())
    }

    /// Whether the game is won: all four foundations complete (Ace..King).
    pub fn is_won(&self) -> bool {
        self.foundations.iter().all(|f| f.is_complete())
    }

    /// All currently legal moves under the classic ruleset.
    pub fn legal_moves(&self) -> Vec<Move> {
        rules::legal_moves(self)
    }

    /// Apply a move, mutating the state. Illegal moves are rejected and leave
    /// the state unchanged.
    pub fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        rules::apply_move(self, mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_classic() {
        let c = GameConfig::default();
        assert_eq!(c.draw_mode, DrawMode::Three);
        assert_eq!(c.redeal_limit, None);
        assert!(!c.timed);
    }

    #[test]
    fn initial_layout_is_correct() {
        let g = GameState::new_with_seed(42, GameConfig::default());
        // Column i has i+1 cards, only the top face-up.
        for (i, col) in g.tableau.iter().enumerate() {
            assert_eq!(col.len(), i + 1, "column {i} size");
            let face_up: Vec<_> = col.cards().iter().filter(|c| c.face_up).collect();
            assert_eq!(
                face_up.len(),
                1,
                "column {i} should have exactly one face-up card"
            );
            assert!(
                col.top().unwrap().face_up,
                "column {i} top should be face-up"
            );
            assert!(col.invariant_holds());
        }
        // 28 dealt to tableau, 24 remain in the stock.
        assert_eq!(g.stock.len(), 24);
        assert!(g.stock.cards().iter().all(|c| !c.face_up));
        assert!(g.waste.is_empty());
        assert!(g.foundations.iter().all(|f| f.is_empty()));
    }

    #[test]
    fn deal_is_reproducible() {
        let a = GameState::new_with_seed(2024, GameConfig::default());
        let b = GameState::new_with_seed(2024, GameConfig::default());
        assert_eq!(a, b);
        // A different seed almost certainly differs.
        let c = GameState::new_with_seed(2025, GameConfig::default());
        assert_ne!(a, c);
    }

    #[test]
    fn state_retains_seed_and_config() {
        let cfg = GameConfig {
            draw_mode: DrawMode::One,
            redeal_limit: Some(3),
            timed: true,
        };
        let g = GameState::new_with_seed(7, cfg);
        assert_eq!(g.seed(), 7);
        assert_eq!(g.config(), cfg);
    }

    #[test]
    fn total_dealt_cards_is_52() {
        let g = GameState::new_with_seed(1, GameConfig::default());
        let tableau_cards: usize = g.tableau.iter().map(|c| c.len()).sum();
        assert_eq!(tableau_cards + g.stock.len() + g.waste.len(), 52);
    }
}
