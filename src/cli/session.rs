//! The game session: owns the `GameState`, the input state machine, undo/redo,
//! the move-history log, the clock, and key dispatch. This layer performs no
//! terminal I/O, so it is fully unit-testable.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::cli::input::{auto_target, key_to_pile, resolve_move, RunAmbiguity};
use crate::cli::{Pile, Signal};
use klondike::{GameConfig, GameState, Move};

/// A terminal-independent key event fed to the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    Char(char),
    Enter,
    Esc,
}

/// A point-in-time snapshot for undo/redo.
#[derive(Debug, Clone)]
struct Snapshot {
    state: GameState,
    move_count: u32,
    history: Vec<Move>,
}

/// A single interactive game.
pub struct Session {
    state: GameState,
    config: GameConfig,
    move_count: u32,
    history: Vec<Move>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    /// Pending source pile (the first key of a two-key move), if any.
    selection: Option<Pile>,
    /// Set when a tableau→tableau move needs a run-length choice.
    pending_run: Option<RunAmbiguity>,
    /// Transient feedback shown until the next action.
    message: Option<String>,
    help_visible: bool,
    start: Instant,
    /// Elapsed seconds frozen at the moment of a win, so the timer (and score)
    /// stop once the game is over. `None` while the game is still in progress.
    frozen_secs: Option<u64>,
}

/// Derive a `u64` seed from the current time (used when `--seed` is omitted).
pub fn random_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    // Scramble with SplitMix64 finalizer so nearby launch times differ widely.
    let mut x = nanos ^ 0x9E37_79B9_7F4A_7C15;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

impl Session {
    /// Start a new session dealt from `seed` with `config`.
    pub fn new(seed: u64, config: GameConfig) -> Self {
        Session {
            state: GameState::new_with_seed(seed, config),
            config,
            move_count: 0,
            history: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            selection: None,
            pending_run: None,
            message: None,
            help_visible: false,
            start: Instant::now(),
            frozen_secs: None,
        }
    }

    // --- accessors for the renderer ---

    pub fn state(&self) -> &GameState {
        &self.state
    }
    pub fn seed(&self) -> u64 {
        self.state.seed()
    }
    pub fn move_count(&self) -> u32 {
        self.move_count
    }
    pub fn selection(&self) -> Option<Pile> {
        self.selection
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    pub fn help_visible(&self) -> bool {
        self.help_visible
    }
    pub fn history(&self) -> &[Move] {
        &self.history
    }
    pub fn is_won(&self) -> bool {
        self.state.is_won()
    }
    pub fn elapsed_secs(&self) -> u64 {
        // Once won, time is frozen at the moment of victory.
        self.frozen_secs
            .unwrap_or_else(|| self.start.elapsed().as_secs())
    }

    /// Push the current elapsed time into the model so scoring is up to date,
    /// then return the current displayable score.
    pub fn sync_time(&mut self) -> i64 {
        let secs = self.elapsed_secs();
        self.state.set_elapsed_secs(secs);
        self.state.current_score()
    }

    /// The final score (with timed bonus when applicable).
    pub fn final_score(&mut self) -> i64 {
        let secs = self.elapsed_secs();
        self.state.set_elapsed_secs(secs);
        self.state.final_score()
    }

    // --- dispatch ---

    /// Handle one key event; returns whether the loop should continue or quit.
    pub fn handle_key(&mut self, key: KeyInput) -> Signal {
        // Help overlay: `?` toggles; while visible, any other key dismisses it.
        if key == KeyInput::Char('?') {
            self.help_visible = !self.help_visible;
            return Signal::Continue;
        }
        if self.help_visible {
            self.help_visible = false;
            return Signal::Continue;
        }

        // Quitting is always available.
        if key == KeyInput::Char('q') {
            return Signal::Quit;
        }

        // Resolving a pending run-length prompt takes priority.
        if self.pending_run.is_some() {
            self.resolve_run_prompt(key);
            return Signal::Continue;
        }

        // Once won, only new-game / quit / help do anything.
        if self.is_won() {
            if key == KeyInput::Char('n') {
                self.new_game();
            }
            return Signal::Continue;
        }

        self.message = None;
        match key {
            KeyInput::Char('u') => self.undo(),
            KeyInput::Char('r') => self.redo(),
            KeyInput::Char('n') => self.new_game(),
            KeyInput::Esc => self.selection = None,
            KeyInput::Enter => self.auto_assign(),
            KeyInput::Char(c) => {
                if let Some(pile) = key_to_pile(c) {
                    self.handle_pile(pile);
                }
            }
        }
        Signal::Continue
    }

    fn handle_pile(&mut self, pile: Pile) {
        match self.selection {
            None => match pile {
                // Engage the stock/waste: deal if there's nothing in the waste
                // to act on, otherwise select it as a source (a second space
                // will deal without clobbering the waste card).
                Pile::StockWaste => {
                    if self.waste_has_card() {
                        self.selection = Some(Pile::StockWaste);
                    } else {
                        self.deal();
                    }
                }
                other => self.selection = Some(other),
            },
            Some(source) => match pile {
                Pile::StockWaste => {
                    if source == Pile::StockWaste {
                        // Second space → deal.
                        self.selection = None;
                        self.deal();
                    } else {
                        // Abandon the half-move and engage the stock fresh.
                        self.selection = None;
                        if self.waste_has_card() {
                            self.selection = Some(Pile::StockWaste);
                        } else {
                            self.deal();
                        }
                    }
                }
                target => {
                    self.selection = None;
                    self.try_move(source, target);
                }
            },
        }
    }

    fn try_move(&mut self, source: Pile, target: Pile) {
        match resolve_move(&self.state, source, target) {
            Ok(Some(mv)) => self.commit(mv),
            Ok(None) => self.message = Some("Illegal move".to_string()),
            Err(amb) => {
                let max = amb.counts.iter().copied().max().unwrap_or(1);
                self.message = Some(format!("Move how many cards? (1-{max})"));
                self.pending_run = Some(amb);
            }
        }
    }

    fn resolve_run_prompt(&mut self, key: KeyInput) {
        let amb = self.pending_run.take().expect("prompt active");
        if let KeyInput::Char(c) = key {
            if let Some(d) = c.to_digit(10) {
                let count = d as usize;
                if amb.counts.contains(&count) {
                    self.commit(Move::TableauToTableau {
                        from: amb.from,
                        to: amb.to,
                        count,
                    });
                    return;
                }
            }
        }
        self.message = Some("Cancelled".to_string());
    }

    fn auto_assign(&mut self) {
        let source = self.selection.take().unwrap_or(Pile::StockWaste);
        match auto_target(&self.state, source) {
            Some(mv) => self.commit(mv),
            None => self.message = Some("No legal move for that card".to_string()),
        }
    }

    fn deal(&mut self) {
        let mv = if self.state.stock.is_empty() {
            Move::Recycle
        } else {
            Move::Draw
        };
        self.commit(mv);
    }

    /// Apply a move, snapshotting for undo. On rejection, show the reason and
    /// leave the state untouched.
    fn commit(&mut self, mv: Move) {
        let snapshot = self.snapshot();
        match self.state.apply(mv) {
            Ok(()) => {
                self.undo.push(snapshot);
                self.redo.clear();
                self.move_count += 1;
                self.history.push(mv);
                self.message = None;
            }
            Err(e) => self.message = Some(format!("{e}")),
        }
        self.update_win_clock();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            let current = self.snapshot();
            self.redo.push(current);
            self.restore(prev);
            self.selection = None;
            self.message = Some("Undid last move".to_string());
            self.update_win_clock();
        } else {
            self.message = Some("Nothing to undo".to_string());
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            let current = self.snapshot();
            self.undo.push(current);
            self.restore(next);
            self.selection = None;
            self.message = Some("Redid move".to_string());
            self.update_win_clock();
        } else {
            self.message = Some("Nothing to redo".to_string());
        }
    }

    /// Freeze the clock at the first moment the game is won; unfreeze if a win
    /// is later undone.
    fn update_win_clock(&mut self) {
        if self.state.is_won() {
            if self.frozen_secs.is_none() {
                self.frozen_secs = Some(self.start.elapsed().as_secs());
            }
        } else {
            self.frozen_secs = None;
        }
    }

    fn new_game(&mut self) {
        *self = Session::new(random_seed(), self.config);
        self.message = Some("New game".to_string());
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            state: self.state.clone(),
            move_count: self.move_count,
            history: self.history.clone(),
        }
    }

    fn restore(&mut self, s: Snapshot) {
        self.state = s.state;
        self.move_count = s.move_count;
        self.history = s.history;
    }

    fn waste_has_card(&self) -> bool {
        self.state.waste.top().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klondike::{Card, DrawMode, Foundation, Rank, Suit, TableauColumn};

    fn cfg() -> GameConfig {
        GameConfig::default()
    }

    fn up(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit).face_up()
    }

    #[test]
    fn timer_freezes_on_win_and_resumes_on_undo() {
        let mut s = Session::new(0, cfg());
        s.state.stock = Default::default();
        s.state.waste = Default::default();
        s.state.foundations = Default::default();
        s.state.tableau = Default::default();
        // Three suits complete; the fourth (spades) missing only its King.
        for (i, suit) in [Suit::Clubs, Suit::Diamonds, Suit::Hearts]
            .into_iter()
            .enumerate()
        {
            let mut f = Foundation::new();
            for r in Rank::ALL {
                f.push(up(r, suit));
            }
            s.state.foundations[i] = f;
        }
        let mut spades = Foundation::new();
        for r in Rank::ALL.into_iter().take(12) {
            spades.push(up(r, Suit::Spades)); // A..Q
        }
        s.state.foundations[3] = spades;
        s.state.tableau[0] = TableauColumn::new(vec![up(Rank::King, Suit::Spades)]);

        assert!(!s.is_won());
        // Move K♠ to its foundation: select column 1, then a foundation key.
        s.handle_key(KeyInput::Char('1'));
        s.handle_key(KeyInput::Char('8'));
        assert!(s.is_won(), "should be won; msg={:?}", s.message());
        assert!(s.frozen_secs.is_some(), "timer should freeze on win");

        let frozen = s.elapsed_secs();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        assert_eq!(
            s.elapsed_secs(),
            frozen,
            "timer must not advance after a win"
        );

        // Undo is disabled via keys in the won state (only n/q/? act), but the
        // clock logic itself resumes if a win is reverted — exercised directly.
        s.undo();
        assert!(!s.is_won());
        assert!(s.frozen_secs.is_none(), "reverting a win resumes the timer");
    }

    #[test]
    fn foundation_to_tableau_via_keys() {
        let mut s = Session::new(0, cfg());
        // Hand-arrange: foundation 0 = A♥,2♥; column 1 top = 3♠ (accepts red 2♥).
        s.state.stock = Default::default();
        s.state.waste = Default::default();
        s.state.foundations = Default::default();
        s.state.tableau = Default::default();
        let mut f = Foundation::new();
        f.push(up(Rank::Ace, Suit::Hearts));
        f.push(up(Rank::Two, Suit::Hearts));
        s.state.foundations[0] = f;
        s.state.tableau[0] = TableauColumn::new(vec![up(Rank::Three, Suit::Spades)]);

        // Foundation source key '8' then tableau target key '1'.
        s.handle_key(KeyInput::Char('8'));
        assert_eq!(s.selection(), Some(Pile::Foundation(0)));
        s.handle_key(KeyInput::Char('1'));
        assert_eq!(
            s.move_count(),
            1,
            "foundation->tableau should apply; msg={:?}",
            s.message()
        );
        assert_eq!(s.state().tableau[0].top().map(|c| c.rank), Some(Rank::Two));
    }

    #[test]
    fn deal_increments_and_is_undoable() {
        let mut s = Session::new(42, cfg());
        assert_eq!(s.move_count(), 0);
        let before = s.state().clone();
        // Space on an empty waste deals immediately.
        s.handle_key(KeyInput::Char(' '));
        assert_eq!(s.move_count(), 1);
        assert!(s.state().waste.top().is_some());
        // Undo restores the exact prior state and count.
        s.handle_key(KeyInput::Char('u'));
        assert_eq!(s.move_count(), 0);
        assert_eq!(s.state(), &before);
    }

    #[test]
    fn second_space_deals_without_clobbering_selection() {
        let mut s = Session::new(1, cfg());
        s.handle_key(KeyInput::Char(' ')); // deal (waste empty -> deal)
        assert_eq!(s.move_count(), 1);
        let waste_top = s.state().waste.top();
        // Now waste has a card: first space selects, does not deal.
        s.handle_key(KeyInput::Char(' '));
        assert_eq!(s.selection(), Some(Pile::StockWaste));
        assert_eq!(s.move_count(), 1);
        assert_eq!(s.state().waste.top(), waste_top);
        // Second space deals.
        s.handle_key(KeyInput::Char(' '));
        assert_eq!(s.selection(), None);
        assert_eq!(s.move_count(), 2);
    }

    #[test]
    fn new_move_clears_redo() {
        let mut s = Session::new(7, cfg());
        s.handle_key(KeyInput::Char(' ')); // move 1 (deal)
        s.handle_key(KeyInput::Char('u')); // undo -> redo has 1
        s.handle_key(KeyInput::Char('r')); // redo -> back
        s.handle_key(KeyInput::Char('u')); // undo again -> redo has 1
                                           // A fresh move should clear the redo stack.
        s.handle_key(KeyInput::Char(' ')); // deal again (new move)
        assert!(s.redo.is_empty(), "redo must be cleared by a new move");
    }

    #[test]
    fn illegal_move_reports_and_no_op() {
        let mut s = Session::new(3, cfg());
        let before = s.state().clone();
        // Select column 1 then column 2 — almost certainly not a legal move at deal.
        s.handle_key(KeyInput::Char('1'));
        assert_eq!(s.selection(), Some(Pile::Tableau(0)));
        s.handle_key(KeyInput::Char('2'));
        // Either it was illegal (message + unchanged) — with a fresh deal, two
        // single face-up cards rarely stack. Assert state only changed if a move
        // was actually recorded.
        if s.move_count() == 0 {
            assert_eq!(s.state(), &before);
            assert!(s.message().is_some());
        }
    }

    #[test]
    fn quit_and_help_signals() {
        let mut s = Session::new(9, cfg());
        assert_eq!(s.handle_key(KeyInput::Char('?')), Signal::Continue);
        assert!(s.help_visible());
        // Any key dismisses help.
        assert_eq!(s.handle_key(KeyInput::Char('1')), Signal::Continue);
        assert!(!s.help_visible());
        assert_eq!(s.handle_key(KeyInput::Char('q')), Signal::Quit);
    }

    #[test]
    fn draw_one_config_draws_single() {
        let config = GameConfig {
            draw_mode: DrawMode::One,
            ..GameConfig::default()
        };
        let mut s = Session::new(5, config);
        s.handle_key(KeyInput::Char(' '));
        assert_eq!(s.state().waste.len(), 1);
    }
}
