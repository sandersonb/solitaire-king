//! Background solvability assist.
//!
//! Drives the solver from the GUI without blocking the frame: it pumps short
//! node-bounded searches that share one persistent transposition table (see
//! `klondike::solve_reusing`), so repeated slices converge while the game stays
//! responsive on native and single-threaded WebAssembly alike. It tracks the
//! current position's status, remembers decided positions (so undo/redo resolve
//! instantly), and owns the non-nagging unwinnable-dialog policy.

use std::collections::HashMap;

use macroquad::prelude::get_time;

use klondike::{encode, solve_reusing, ClosedTable, GameState, PositionKey, SolveOptions, Verdict};

/// Wall-clock cap on a single background check (seconds of solver work).
const CHECK_BUDGET_SECS: f64 = 1.0;
/// Idle time before a check is scheduled for the current position.
const IDLE_SECS: f64 = 3.0;
/// Nodes expanded per per-frame slice. Small enough to keep the frame smooth;
/// the table makes repeated slices cumulative. Tunable.
const SLICE_NODES: u64 = 6_000;
/// Transposition-table entry cap (exact-byte keys). Eviction is sound.
const TABLE_CAP: usize = 1 << 20;

/// The current position's solvability, as shown by the indicator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// Not yet checked / status unknown.
    Unknown,
    /// A background check is running.
    Checking,
    /// A winning line exists.
    Solvable,
    /// Proven no win is reachable.
    Unwinnable,
    /// A check ended without a decisive answer.
    Inconclusive,
}

struct ActiveCheck {
    /// The position being checked (guards against the board changing mid-check).
    key: PositionKey,
    /// Cumulative solver time spent on this check.
    spent: f64,
}

pub struct Assist {
    /// Persistent proven-winless knowledge, reused across checks and moves.
    table: ClosedTable<PositionKey>,
    /// Decided positions, so revisits/undo resolve without a new search.
    decided: HashMap<PositionKey, Verdict>,
    options: SolveOptions,
    status: Status,
    check: Option<ActiveCheck>,
    last_activity: f64,
    dialog_open: bool,
    /// Suppresses the dialog after the player chose Continue, until solvability
    /// returns (status leaves Unwinnable).
    dismissed_streak: bool,
}

impl Assist {
    /// Create the assist for a fresh deal and start the opening check.
    pub fn new(state: &GameState) -> Self {
        let mut a = Assist {
            table: ClosedTable::with_capacity(TABLE_CAP),
            decided: HashMap::new(),
            options: SolveOptions::default(),
            status: Status::Unknown,
            check: None,
            last_activity: get_time(),
            dialog_open: false,
            dismissed_streak: false,
        };
        a.begin_check(state);
        a
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn dialog_open(&self) -> bool {
        self.dialog_open
    }

    /// Reset for a new deal: drop knowledge and start a fresh opening check.
    pub fn reset(&mut self, state: &GameState) {
        self.table = ClosedTable::with_capacity(TABLE_CAP);
        self.decided.clear();
        self.dismissed_streak = false;
        self.dialog_open = false;
        self.check = None;
        self.last_activity = get_time();
        self.begin_check(state);
    }

    /// A move/undo/redo changed the position: record activity, abandon any
    /// in-flight check, and re-resolve status for the new position.
    pub fn on_state_change(&mut self, state: &GameState) {
        self.last_activity = get_time();
        self.check = None;
        let key = encode(state);
        match self.decided.get(&key).copied() {
            Some(v) => self.set_status_from_verdict(v),
            // A not-yet-checked position is Unknown; do NOT touch the dismissed
            // streak here — forward play from an unwinnable position transiently
            // passes through Unknown, and clearing the streak there would let the
            // dialog re-open on the next check. The streak is re-armed only when
            // solvability genuinely returns (status becomes Solvable).
            None => self.status = Status::Unknown,
        }
    }

    /// Note non-move interaction (e.g. pointer activity) to reset the idle timer.
    pub fn note_activity(&mut self) {
        self.last_activity = get_time();
    }

    /// The player chose Continue on the unwinnable dialog.
    pub fn dismiss_dialog(&mut self) {
        self.dialog_open = false;
        self.dismissed_streak = true;
    }

    /// Per-frame: pump the active check a slice, or schedule one when idle.
    pub fn update(&mut self, state: &GameState) {
        // Abandon a check whose position no longer matches the board.
        if let Some(c) = self.check.as_ref() {
            if c.key != encode(state) {
                self.check = None;
            }
        }

        if let Some(mut check) = self.check.take() {
            let t0 = get_time();
            let r = solve_reusing(state, Some(SLICE_NODES), self.options, &mut self.table);
            check.spent += (get_time() - t0).max(0.0);
            match r.verdict {
                Verdict::Solvable => {
                    self.decided.insert(check.key, Verdict::Solvable);
                    self.last_activity = get_time();
                    self.set_status_from_verdict(Verdict::Solvable);
                }
                Verdict::Unwinnable => {
                    self.decided.insert(check.key, Verdict::Unwinnable);
                    self.last_activity = get_time();
                    self.set_status_from_verdict(Verdict::Unwinnable);
                }
                Verdict::Inconclusive => {
                    if check.spent >= CHECK_BUDGET_SECS {
                        self.status = Status::Inconclusive;
                        self.last_activity = get_time();
                    } else {
                        self.check = Some(check); // keep pumping next frame
                    }
                }
            }
            return;
        }

        // Idle: (re-)check an uncertain position. Known-decided positions are not
        // re-searched (they resolved via `decided` on the state change).
        if matches!(self.status, Status::Unknown | Status::Inconclusive)
            && get_time() - self.last_activity > IDLE_SECS
        {
            self.begin_check(state);
        }
    }

    /// Start (or instantly resolve) a check of `state`.
    fn begin_check(&mut self, state: &GameState) {
        let key = encode(state);
        if let Some(v) = self.decided.get(&key).copied() {
            self.set_status_from_verdict(v); // known: no new search (req 6)
            return;
        }
        self.check = Some(ActiveCheck { key, spent: 0.0 });
        self.status = Status::Checking;
    }

    fn set_status_from_verdict(&mut self, v: Verdict) {
        self.status = match v {
            Verdict::Solvable => Status::Solvable,
            Verdict::Unwinnable => Status::Unwinnable,
            Verdict::Inconclusive => Status::Inconclusive,
        };
        match self.status {
            // Solvability returned: re-arm the dialog for a future unwinnable
            // streak (e.g. after an undo back to a winnable position).
            Status::Solvable => self.dismissed_streak = false,
            // Only warn once per unwinnable streak; Continue set the flag.
            Status::Unwinnable if !self.dismissed_streak => self.dialog_open = true,
            _ => {}
        }
    }
}
