//! Microsoft Windows Solitaire "Standard" scoring.
//!
//! Point values are named constants so the exact rules are auditable in one
//! place. The scoring table itself is pure: the *rules engine* decides which
//! events fire (e.g. whether a recycle is penalized), while this module only
//! knows what each event is worth and keeps the running total.

/// Waste → tableau: +5.
pub const SCORE_WASTE_TO_TABLEAU: i32 = 5;
/// Turning over (flipping) a tableau card: +5.
pub const SCORE_FLIP_TABLEAU: i32 = 5;
/// Waste → foundation: +10.
pub const SCORE_WASTE_TO_FOUNDATION: i32 = 10;
/// Tableau → foundation: +10.
pub const SCORE_TABLEAU_TO_FOUNDATION: i32 = 10;
/// Foundation → tableau: −15.
pub const SCORE_FOUNDATION_TO_TABLEAU: i32 = -15;
/// Draw-one recycle penalty (per pass after the first): −100.
pub const RECYCLE_PENALTY_DRAW_ONE: i32 = 100;

/// Timed mode: points deducted per full 10 seconds of play.
pub const TIME_PENALTY_PER_10S: i64 = 2;
/// Timed mode: numerator of the win-time bonus (`bonus = TIME_BONUS_NUMERATOR / seconds`).
pub const TIME_BONUS_NUMERATOR: i64 = 700_000;

/// A discrete scoring event caused by applying a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreEvent {
    WasteToTableau,
    FlipTableauCard,
    WasteToFoundation,
    TableauToFoundation,
    FoundationToTableau,
    /// A draw-one recycle after the first pass (draw-three recycles do not emit this).
    RecycleDrawOne,
}

impl ScoreEvent {
    /// The signed point delta this event contributes.
    pub fn points(self) -> i32 {
        match self {
            ScoreEvent::WasteToTableau => SCORE_WASTE_TO_TABLEAU,
            ScoreEvent::FlipTableauCard => SCORE_FLIP_TABLEAU,
            ScoreEvent::WasteToFoundation => SCORE_WASTE_TO_FOUNDATION,
            ScoreEvent::TableauToFoundation => SCORE_TABLEAU_TO_FOUNDATION,
            ScoreEvent::FoundationToTableau => SCORE_FOUNDATION_TO_TABLEAU,
            ScoreEvent::RecycleDrawOne => -RECYCLE_PENALTY_DRAW_ONE,
        }
    }
}

/// The running score. Event points accumulate here and are clamped so they
/// never drop below zero. Time penalty and win bonus are computed from injected
/// elapsed seconds so the core stays deterministic (the front-end owns the clock).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Score {
    event_points: i64,
}

impl Score {
    /// A fresh score of zero.
    pub fn new() -> Self {
        Score { event_points: 0 }
    }

    /// The accumulated event points (never negative), excluding timed effects.
    pub fn event_points(&self) -> i64 {
        self.event_points
    }

    /// Apply a scoring event, clamping the running total at zero.
    pub fn apply(&mut self, event: ScoreEvent) {
        self.event_points = (self.event_points + event.points() as i64).max(0);
    }

    /// The elapsed-time penalty for `elapsed_secs` in timed mode: 2 points per
    /// full 10 seconds.
    pub fn time_penalty(elapsed_secs: u64) -> i64 {
        (elapsed_secs / 10) as i64 * TIME_PENALTY_PER_10S
    }

    /// The win-time bonus for `elapsed_secs` in timed mode. Larger for faster
    /// completions; zero if no time has elapsed.
    pub fn win_bonus(elapsed_secs: u64) -> i64 {
        if elapsed_secs == 0 {
            0
        } else {
            TIME_BONUS_NUMERATOR / elapsed_secs as i64
        }
    }

    /// The current displayable score. In timed mode the elapsed-time penalty is
    /// subtracted; the result is clamped at zero. In untimed mode elapsed time
    /// has no effect.
    pub fn current(&self, elapsed_secs: u64, timed: bool) -> i64 {
        if timed {
            (self.event_points - Self::time_penalty(elapsed_secs)).max(0)
        } else {
            self.event_points
        }
    }

    /// The final score, including the timed win bonus when the game is won in
    /// timed mode.
    pub fn final_score(&self, elapsed_secs: u64, timed: bool, won: bool) -> i64 {
        let mut score = self.current(elapsed_secs, timed);
        if timed && won {
            score += Self::win_bonus(elapsed_secs);
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_point_values() {
        assert_eq!(ScoreEvent::WasteToTableau.points(), 5);
        assert_eq!(ScoreEvent::FlipTableauCard.points(), 5);
        assert_eq!(ScoreEvent::WasteToFoundation.points(), 10);
        assert_eq!(ScoreEvent::TableauToFoundation.points(), 10);
        assert_eq!(ScoreEvent::FoundationToTableau.points(), -15);
        assert_eq!(ScoreEvent::RecycleDrawOne.points(), -100);
    }

    #[test]
    fn score_clamps_at_zero() {
        let mut s = Score::new();
        s.apply(ScoreEvent::FoundationToTableau); // -15, clamped
        assert_eq!(s.event_points(), 0);
        s.apply(ScoreEvent::WasteToFoundation); // +10
        assert_eq!(s.event_points(), 10);
        s.apply(ScoreEvent::FoundationToTableau); // 10 - 15 -> clamp 0
        assert_eq!(s.event_points(), 0);
    }

    #[test]
    fn time_penalty_and_bonus() {
        assert_eq!(Score::time_penalty(20), 4);
        assert_eq!(Score::time_penalty(9), 0);
        assert_eq!(Score::time_penalty(19), 2);
        assert!(Score::win_bonus(60) > 0);
        assert_eq!(Score::win_bonus(0), 0);
    }

    #[test]
    fn untimed_ignores_elapsed() {
        let mut s = Score::new();
        s.apply(ScoreEvent::WasteToFoundation); // 10
        assert_eq!(s.current(1000, false), 10);
        assert_eq!(s.current(1000, true), 0); // penalty exceeds points, clamped
    }

    #[test]
    fn final_score_adds_win_bonus_when_timed() {
        let mut s = Score::new();
        for _ in 0..30 {
            s.apply(ScoreEvent::TableauToFoundation); // 300
        }
        let untimed = s.final_score(60, false, true);
        assert_eq!(untimed, 300);
        let timed = s.final_score(60, true, true);
        // 300 - (60/10)*2 + 700000/60 = 300 - 12 + 11666
        assert_eq!(timed, 300 - 12 + 700_000 / 60);
    }
}
