//! Card motion tweens. When a move changes where a card rests, the game state
//! is updated immediately (see the input handler) and a short animation slides
//! the card(s) from where they were released to the pile's resting position.
//! Animation is purely cosmetic — input and scoring never wait on it.
//!
//! The same queue can play back a sequence of moves without pointer input
//! (`enqueue_moves`), which is the seam a future automated/solver playback uses.

use macroquad::prelude::*;

use crate::input::Pile;
use klondike::{Card, Move};

/// One in-flight card (or run) sliding from `from` to `to`.
pub struct CardAnim {
    /// The card(s) carried, drawn as a small downward fan like a tableau run.
    pub cards: Vec<Card>,
    pub from: Vec2,
    pub to: Vec2,
    pub fan_dy: f32,
    pub card_w: f32,
    pub start: f64,
    pub dur: f64,
    /// Destination cards to suppress in the static board until this lands, so a
    /// card isn't drawn both at rest and in flight. `None` for a return-to-origin.
    pub hide: Option<(Pile, usize)>,
}

impl CardAnim {
    /// Progress in `0..=1` with a smoothstep ease.
    fn progress(&self, now: f64) -> f32 {
        let t = if self.dur <= 0.0 {
            1.0
        } else {
            ((now - self.start) / self.dur).clamp(0.0, 1.0) as f32
        };
        t * t * (3.0 - 2.0 * t)
    }

    /// Current top-left of the primary (bottom) card.
    pub fn pos(&self, now: f64) -> Vec2 {
        let e = self.progress(now);
        self.from + (self.to - self.from) * e
    }

    fn done(&self, now: f64) -> bool {
        now - self.start >= self.dur
    }
}

/// Default snap duration (seconds).
pub const SNAP_SECS: f64 = 0.14;

#[derive(Default)]
pub struct Animator {
    pub anims: Vec<CardAnim>,
    /// Moves queued for automated playback (drained by the main loop when idle).
    queue: Vec<Move>,
}

impl Animator {
    pub fn new() -> Self {
        Animator::default()
    }

    /// Start a card animation.
    pub fn push(&mut self, anim: CardAnim) {
        self.anims.push(anim);
    }

    /// Drop finished animations. Call once per frame.
    pub fn tick(&mut self, now: f64) {
        self.anims.retain(|a| !a.done(now));
    }

    pub fn is_animating(&self) -> bool {
        !self.anims.is_empty()
    }

    /// How many top cards of `pile` are currently animating in (and so should be
    /// hidden in the static board render).
    pub fn suppressed(&self, pile: Pile) -> usize {
        self.anims
            .iter()
            .filter_map(|a| a.hide)
            .filter(|(p, _)| *p == pile)
            .map(|(_, n)| n)
            .sum()
    }

    /// Queue a sequence of moves for automated playback (e.g. animating a
    /// solver's solution). The main loop applies and animates them in order when
    /// no drag or animation is in flight. Reserved for a future auto-play/solver
    /// feature; not yet driven by the UI.
    #[allow(dead_code)]
    pub fn enqueue_moves(&mut self, moves: &[Move]) {
        self.queue.extend_from_slice(moves);
    }

    /// Pop the next queued playback move, if any.
    pub fn next_queued(&mut self) -> Option<Move> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }
}
