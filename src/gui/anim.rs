//! Card motion tweens. When a move changes where a card rests, the game state
//! is updated immediately (see the input handler) and a short animation slides
//! the card(s) from where they were released to the pile's resting position.
//! Animation is purely cosmetic — input and scoring never wait on it.
//!
//! The same queue can play back a sequence of moves without pointer input
//! (`enqueue_moves`), which is the seam a future automated/solver playback uses.

use macroquad::prelude::*;
use macroquad::rand::{gen_range, srand};

use crate::input::Pile;
use klondike::{Card, Foundation, Move, Rank};

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

// ---- Win celebration ------------------------------------------------------
//
// A physics cascade played on a won game: the 52 foundation cards fall (Kings
// first, down through the ranks to the Aces), accelerate under gravity, bounce
// off the floor and walls with random kicks and rotation, and settle. Purely
// cosmetic; runs ~20s or until the player dismisses it.

/// Total celebration length before it ends on its own (seconds). The cascade
/// settles within ~6s, so a longer cap only made the player wait.
pub const CELEBRATION_SECS: f64 = 6.0;
/// Downward acceleration (px/s²); large so cards fall fast enough for ~20s.
const GRAVITY: f32 = 2600.0;
/// Delay between successive rank batches launching (seconds); 13 ranks ≈ 3.25s.
const LAUNCH_INTERVAL: f64 = 0.25;
/// Bounce energy retained on a floor hit (randomized per bounce).
const REST_MIN: f32 = 0.45;
const REST_MAX: f32 = 0.70;
/// Horizontal velocity kick added on each floor bounce (± px/s).
const KICK: f32 = 350.0;
/// Angular velocity given on a bounce (± rad/s).
const ANG_MAX: f32 = 6.0;
/// Fraction of horizontal velocity retained on a wall bounce.
const WALL_DAMP: f32 = 0.6;
/// Below this downward speed at a floor contact, the card comes to rest (px/s).
const REST_EPS: f32 = 60.0;
/// Per-card gravity jitter range.
const G_SCALE_MIN: f32 = 0.85;
const G_SCALE_MAX: f32 = 1.15;
/// Frame-delta clamp so a long stall (backgrounded tab) can't tunnel cards.
const DT_MAX: f32 = 1.0 / 30.0;

/// One card in the celebration cascade.
pub struct FallingCard {
    pub card: Card,
    /// Which foundation (0..4) this card launched from.
    foundation: usize,
    /// Top-left in screen pixels.
    pub pos: Vec2,
    pub vel: Vec2,
    /// Rotation in radians (drawn about the card center).
    pub rot: f32,
    ang_vel: f32,
    g_scale: f32,
    /// Seconds after the celebration start when this card begins to fall.
    launch_at: f64,
    launched: bool,
    at_rest: bool,
}

impl FallingCard {
    /// Whether this card has started falling (and so should be drawn).
    pub fn visible(&self) -> bool {
        self.launched
    }
}

/// The win celebration state: all 52 cards plus the sizing/deck needed to draw
/// them. Advanced by frame delta; ended by the main loop on click or timeout.
pub struct Celebration {
    cards: Vec<FallingCard>,
    card_w: f32,
    card_h: f32,
    mobile_deck: bool,
    started: f64,
    /// Set once the animation has ended (click or timeout). The settled cards keep
    /// being drawn — under the win banner — until a new game clears them.
    finished: bool,
}

impl Celebration {
    /// Build the cascade from the (won) foundations: for each rank King→Ace, the
    /// four suits launch together, seeded at their foundation's screen position.
    pub fn start(
        foundations: &[Foundation; 4],
        rects: &[Rect; 4],
        card_w: f32,
        card_h: f32,
        mobile_deck: bool,
        now: f64,
    ) -> Celebration {
        // Vary each celebration a little.
        srand(now.to_bits());
        let mut cards = Vec::with_capacity(52);
        // Kings first, down to Aces: enumerate rank batches so `launch_at` grows.
        for (batch, rank) in Rank::ALL.iter().rev().enumerate() {
            let launch_at = batch as f64 * LAUNCH_INTERVAL;
            for (i, f) in foundations.iter().enumerate() {
                if let Some(card) = f.cards().iter().find(|c| c.rank == *rank).copied() {
                    let r = rects[i];
                    cards.push(FallingCard {
                        card,
                        foundation: i,
                        pos: vec2(r.x, r.y),
                        vel: Vec2::ZERO,
                        rot: 0.0,
                        ang_vel: 0.0,
                        g_scale: gen_range(G_SCALE_MIN, G_SCALE_MAX),
                        launch_at,
                        launched: false,
                        at_rest: false,
                    });
                }
            }
        }
        Celebration {
            cards,
            card_w,
            card_h,
            mobile_deck,
            started: now,
            finished: false,
        }
    }

    /// Seconds since the celebration started.
    pub fn elapsed(&self, now: f64) -> f64 {
        now - self.started
    }

    /// The cards to draw (only launched ones are visible).
    pub fn cards(&self) -> &[FallingCard] {
        &self.cards
    }

    /// The card still resting on `foundation` — the highest-rank one not yet
    /// launched — so the board can paint the "next card on the deck" as the one
    /// above it falls. `None` once every card of that foundation has launched.
    pub fn resting_top(&self, foundation: usize) -> Option<Card> {
        // Cards are stored King→Ace, so the first not-yet-launched card for this
        // foundation is its current top.
        self.cards
            .iter()
            .find(|c| c.foundation == foundation && !c.launched)
            .map(|c| c.card)
    }
    pub fn card_w(&self) -> f32 {
        self.card_w
    }
    pub fn card_h(&self) -> f32 {
        self.card_h
    }
    pub fn mobile_deck(&self) -> bool {
        self.mobile_deck
    }

    /// Advance the physics one frame within a `sw × sh` screen.
    pub fn update(&mut self, dt: f32, sw: f32, sh: f32, now: f64) {
        let dt = dt.min(DT_MAX);
        let elapsed = self.elapsed(now);
        let floor = (sh - self.card_h).max(0.0);
        let right = (sw - self.card_w).max(0.0);
        for c in &mut self.cards {
            if !c.launched {
                if elapsed >= c.launch_at {
                    c.launched = true;
                    // Drop straight down (tiny random downward nudge only).
                    c.vel = vec2(0.0, gen_range(0.0, 60.0));
                } else {
                    continue;
                }
            }
            if c.at_rest {
                continue;
            }
            c.vel.y += GRAVITY * c.g_scale * dt;
            c.pos += c.vel * dt;
            c.rot += c.ang_vel * dt;

            // Walls: reflect and damp so cards stay on screen (done before the
            // floor/settle branch, which may `continue`).
            if c.pos.x < 0.0 {
                c.pos.x = 0.0;
                c.vel.x = -c.vel.x * WALL_DAMP;
            } else if c.pos.x > right {
                c.pos.x = right;
                c.vel.x = -c.vel.x * WALL_DAMP;
            }

            // Floor: bounce with decay, random horizontal kick and spin, or settle.
            if c.pos.y >= floor {
                c.pos.y = floor;
                if c.vel.y.abs() < REST_EPS {
                    c.at_rest = true;
                    c.vel = Vec2::ZERO;
                    c.ang_vel = 0.0;
                    continue;
                }
                c.vel.y = -c.vel.y * gen_range(REST_MIN, REST_MAX);
                c.vel.x += gen_range(-KICK, KICK);
                c.ang_vel = gen_range(-ANG_MAX, ANG_MAX);
            }
        }
    }
}

#[derive(Default)]
pub struct Animator {
    pub anims: Vec<CardAnim>,
    /// Moves queued for automated playback (drained by the main loop when idle).
    queue: Vec<Move>,
    /// Earliest time the next queued move may start (paces auto-solve playback).
    next_at: f64,
    /// The win celebration, while one is playing.
    celebration: Option<Celebration>,
}

/// Delay between successive auto-played moves (seconds).
pub const PLAY_SECS: f64 = 0.5;

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

    /// Queue a sequence of moves for automated playback (auto-solve). They are
    /// applied and animated in order, paced by `PLAY_SECS`.
    pub fn enqueue_moves(&mut self, moves: &[Move], now: f64) {
        self.queue.extend_from_slice(moves);
        self.next_at = now; // the first move may play immediately
    }

    /// Whether any queued playback moves remain.
    pub fn has_queued(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Discard any queued playback moves (e.g. to cancel an auto-solve).
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Pop the next queued move if the pacing interval has elapsed, advancing the
    /// next-allowed time by `PLAY_SECS`.
    pub fn take_next(&mut self, now: f64) -> Option<Move> {
        if self.queue.is_empty() || now < self.next_at {
            return None;
        }
        self.next_at = now + PLAY_SECS;
        Some(self.queue.remove(0))
    }

    /// Begin the win celebration cascade.
    pub fn start_celebration(&mut self, c: Celebration) {
        self.celebration = Some(c);
    }

    /// Whether the win celebration is actively playing (physics running, banner
    /// hidden). False once it has finished, even though the settled cards remain.
    pub fn celebration_active(&self) -> bool {
        self.celebration.as_ref().is_some_and(|c| !c.finished)
    }

    /// The celebration state, for rendering (present while playing *and* after it
    /// finishes, until a new game clears it).
    pub fn celebration(&self) -> Option<&Celebration> {
        self.celebration.as_ref()
    }

    /// Advance the celebration a frame, if one is playing.
    pub fn update_celebration(&mut self, dt: f32, sw: f32, sh: f32, now: f64) {
        if let Some(c) = &mut self.celebration {
            c.update(dt, sw, sh, now);
        }
    }

    /// Stop the animation (dismissed or timed out) but keep the settled cards on
    /// screen; the win banner then shows over them.
    pub fn finish_celebration(&mut self) {
        if let Some(c) = &mut self.celebration {
            c.finished = true;
        }
    }

    /// Clear the celebration entirely, removing its cards (on a new game).
    pub fn end_celebration(&mut self) {
        self.celebration = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klondike::Suit;

    /// Four full foundations (each suit A..K), as at a win.
    fn won_foundations() -> [Foundation; 4] {
        let mut fs: [Foundation; 4] = Default::default();
        for (i, suit) in Suit::ALL.iter().enumerate() {
            for rank in Rank::ALL {
                fs[i].push(Card::new(rank, *suit).face_up());
            }
        }
        fs
    }

    fn rects() -> [Rect; 4] {
        [
            Rect::new(100.0, 20.0, 80.0, 112.0),
            Rect::new(200.0, 20.0, 80.0, 112.0),
            Rect::new(300.0, 20.0, 80.0, 112.0),
            Rect::new(400.0, 20.0, 80.0, 112.0),
        ]
    }

    #[test]
    fn schedule_has_all_52_kings_first_to_aces_last() {
        let c = Celebration::start(&won_foundations(), &rects(), 80.0, 112.0, false, 0.0);
        assert_eq!(c.cards().len(), 52, "all 52 cards scheduled");
        // First batch (4) are Kings, last batch (4) are Aces.
        assert!(c.cards()[..4].iter().all(|f| f.card.rank == Rank::King));
        assert!(c.cards()[48..].iter().all(|f| f.card.rank == Rank::Ace));
        // launch_at is non-decreasing (Kings launch earliest).
        assert!(c.cards().windows(2).all(|w| w[0].launch_at <= w[1].launch_at));
        assert!(c.cards()[0].launch_at < c.cards()[51].launch_at);
    }

    #[test]
    fn all_cards_settle_within_bounds() {
        let (sw, sh, cw, ch) = (960.0f32, 800.0f32, 80.0f32, 112.0f32);
        let mut c = Celebration::start(&won_foundations(), &rects(), cw, ch, false, 0.0);
        // Simulate 30s at 60fps — beyond the 20s cap, so everything must settle.
        let dt = 1.0 / 60.0;
        let mut now = 0.0f64;
        for _ in 0..(30 * 60) {
            now += dt as f64;
            c.update(dt, sw, sh, now);
        }
        for f in c.cards() {
            assert!(f.launched, "every card launched within 30s");
            assert!(f.at_rest, "every card came to rest");
            assert!(
                f.pos.x >= 0.0 && f.pos.x <= sw - cw,
                "x in bounds: {}",
                f.pos.x
            );
            assert!(f.pos.y <= sh - ch + 0.5, "y at/above floor: {}", f.pos.y);
        }
    }
}
