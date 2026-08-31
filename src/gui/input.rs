//! Pointer input (mouse + touch), hit-testing, drag state, and (pure) move
//! resolution over the model.

use macroquad::prelude::*;

use crate::layout::{Layout, NUM_TABLEAU};
use klondike::{legal_moves, Card, GameState, Move};

/// A destination pile a move can target (also used as a suppression key while a
/// card animates into or back out of a pile).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pile {
    Tableau(usize),
    Foundation(usize),
    Waste,
}

/// A selected move source.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Waste,
    Foundation(usize),
    /// A tableau column with the index of the clicked (bottom-of-run) card.
    TableauRun {
        col: usize,
        index: usize,
    },
}

/// What the mouse is currently over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hit {
    Stock,
    Waste,
    Foundation(usize),
    /// A specific face-up card in a column.
    TableauCard {
        col: usize,
        index: usize,
    },
    /// An empty tableau column.
    TableauEmpty(usize),
    None,
}

fn contains(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h
}

/// A unified pointer sampled from touch (preferred) or the mouse, so the drag
/// logic is input-source agnostic and a touch tap isn't handled twice on web.
#[derive(Clone, Copy, Default)]
pub struct Pointer {
    pub x: f32,
    pub y: f32,
    /// Went down this frame.
    pub pressed: bool,
    /// Held down this frame (for press-and-hold detection).
    pub down: bool,
    /// Released this frame.
    pub released: bool,
}

/// Read the current pointer, preferring an active touch over the mouse.
pub fn read_pointer() -> Pointer {
    if let Some(t) = touches().into_iter().next() {
        let (pressed, down, released) = match t.phase {
            TouchPhase::Started => (true, true, false),
            TouchPhase::Moved | TouchPhase::Stationary => (false, true, false),
            TouchPhase::Ended | TouchPhase::Cancelled => (false, false, true),
        };
        // macroquad's `touches()` returns positions in *physical* pixels (unlike
        // `mouse_position()` and `screen_width()`, which are divided by the DPI
        // scale). On a Retina/iOS display that mismatch throws touch hit-testing
        // off by the DPI factor, so convert to the same logical space here.
        let dpi = screen_dpi_scale();
        Pointer {
            x: t.position.x / dpi,
            y: t.position.y / dpi,
            pressed,
            down,
            released,
        }
    } else {
        let (x, y) = mouse_position();
        Pointer {
            x,
            y,
            pressed: is_mouse_button_pressed(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
            released: is_mouse_button_released(MouseButton::Left),
        }
    }
}

/// A card (or run) currently being dragged by the pointer.
pub struct Drag {
    pub source: Source,
    /// The face-up run being carried, bottom-first (for drawing).
    pub cards: Vec<Card>,
    /// Offset from the grabbed card's top-left to the pointer, so it tracks 1:1.
    pub grab_dx: f32,
    pub grab_dy: f32,
    pub pos: Vec2,
    /// Original top-left of the grabbed (bottom) card, for a return animation.
    pub origin: Vec2,
}

impl Drag {
    /// Current top-left of the grabbed (bottom) card.
    pub fn top_left(&self) -> Vec2 {
        vec2(self.pos.x - self.grab_dx, self.pos.y - self.grab_dy)
    }
}

/// The drop zone (destination pile) nearest to point `(x, y)`, if one is within
/// reach. Zones are the foundation and tableau rects padded generously, so a
/// release need not land exactly on the pile.
pub fn nearest_pile(state: &GameState, layout: &Layout, x: f32, y: f32) -> Option<Pile> {
    let pad = layout.card_w * 0.5;
    let card_h = layout.card_w * 1.4;
    let mut best: Option<(Pile, f32)> = None;
    let mut consider = |pile: Pile, r: Rect| {
        let expanded = Rect::new(r.x - pad, r.y - pad, r.w + 2.0 * pad, r.h + 2.0 * pad);
        if contains(expanded, x, y) {
            let cx = r.x + r.w / 2.0;
            let cy = r.y + r.h / 2.0;
            let d = (x - cx).powi(2) + (y - cy).powi(2);
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((pile, d));
            }
        }
    };
    for (i, r) in layout.foundations.iter().enumerate() {
        consider(Pile::Foundation(i), *r);
    }
    for col in 0..NUM_TABLEAU {
        let len = state.tableau[col].len();
        let base = layout.tableau[col];
        let h = if len <= 1 {
            card_h
        } else {
            card_h + layout.fan_dy * (len as f32 - 1.0)
        };
        consider(Pile::Tableau(col), Rect::new(base.x, base.y, base.w, h));
    }
    best.map(|(p, _)| p)
}

/// Determine what pile/card the point `(x, y)` is over.
pub fn hit_test(state: &GameState, layout: &Layout, x: f32, y: f32) -> Hit {
    if contains(layout.stock, x, y) {
        return Hit::Stock;
    }
    // The waste fans up to three cards to the right; accept the whole fan.
    let waste_hit = Rect::new(
        layout.waste.x,
        layout.waste.y,
        layout.waste.w * 1.6,
        layout.waste.h,
    );
    if contains(waste_hit, x, y) {
        return Hit::Waste;
    }
    for (i, r) in layout.foundations.iter().enumerate() {
        if contains(*r, x, y) {
            return Hit::Foundation(i);
        }
    }
    for col in 0..NUM_TABLEAU {
        let n = state.tableau[col].len();
        if n == 0 {
            if contains(layout.tableau[col], x, y) {
                return Hit::TableauEmpty(col);
            }
            continue;
        }
        // Topmost card wins: scan from the top card downward.
        for index in (0..n).rev() {
            if contains(layout.tableau_card_rect(col, index), x, y) {
                return Hit::TableauCard { col, index };
            }
        }
    }
    Hit::None
}

/// The source a hit selects, if it can be a move origin (face-up cards only).
pub fn source_of(state: &GameState, hit: Hit) -> Option<Source> {
    match hit {
        Hit::Waste if state.waste.top().is_some() => Some(Source::Waste),
        Hit::Foundation(i) if state.foundations[i].top().is_some() => Some(Source::Foundation(i)),
        Hit::TableauCard { col, index } => {
            let card = state.tableau[col].cards().get(index)?;
            card.face_up.then_some(Source::TableauRun { col, index })
        }
        _ => None,
    }
}

/// The destination pile and number of cards a move delivers, for animating it
/// into place and hiding the resting cards until they land. `Draw`/`Recycle`
/// have no card destination and return `None`.
pub fn move_dest(mv: &Move) -> Option<(Pile, usize)> {
    match *mv {
        Move::WasteToFoundation { foundation } => Some((Pile::Foundation(foundation), 1)),
        Move::TableauToFoundation { foundation, .. } => Some((Pile::Foundation(foundation), 1)),
        Move::WasteToTableau { column } => Some((Pile::Tableau(column), 1)),
        Move::FoundationToTableau { column, .. } => Some((Pile::Tableau(column), 1)),
        Move::TableauToTableau { to, count, .. } => Some((Pile::Tableau(to), count)),
        Move::Draw | Move::Recycle => None,
    }
}

/// Resolve a `source` → `dest` selection into a concrete legal move, if any.
pub fn resolve(state: &GameState, source: Source, dest: Pile) -> Option<Move> {
    let legal = legal_moves(state);
    match (source, dest) {
        (Source::Waste, Pile::Foundation(_)) => legal
            .into_iter()
            .find(|m| matches!(m, Move::WasteToFoundation { .. })),
        (Source::Waste, Pile::Tableau(k)) => legal
            .into_iter()
            .find(|m| matches!(m, Move::WasteToTableau { column } if *column == k)),
        (Source::Foundation(i), Pile::Tableau(k)) => legal.into_iter().find(
            |m| matches!(m, Move::FoundationToTableau { foundation, column } if *foundation == i && *column == k),
        ),
        (Source::TableauRun { col, index }, Pile::Tableau(k)) => {
            let count = state.tableau[col].len() - index;
            legal.into_iter().find(
                |m| matches!(m, Move::TableauToTableau { from, to, count: c } if *from == col && *to == k && *c == count),
            )
        }
        (Source::TableauRun { col, index }, Pile::Foundation(_)) => {
            // Only the single top card of a column can go to a foundation.
            if index + 1 == state.tableau[col].len() {
                legal
                    .into_iter()
                    .find(|m| matches!(m, Move::TableauToFoundation { column, .. } if *column == col))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// The best legal move for `source`, preferring a foundation (auto-move).
pub fn auto_target(state: &GameState, source: Source) -> Option<Move> {
    let priority = |m: &Move| -> Option<u8> {
        match (source, m) {
            (Source::Waste, Move::WasteToFoundation { .. }) => Some(0),
            (Source::Waste, Move::WasteToTableau { .. }) => Some(1),
            (Source::Foundation(i), Move::FoundationToTableau { foundation, .. })
                if *foundation == i =>
            {
                Some(2)
            }
            (Source::TableauRun { col, index }, Move::TableauToFoundation { column, .. })
                if *column == col && index + 1 == state.tableau[col].len() =>
            {
                Some(0)
            }
            (Source::TableauRun { col, index }, Move::TableauToTableau { from, count, .. })
                if *from == col && *count == state.tableau[col].len() - index =>
            {
                Some(1)
            }
            _ => None,
        }
    };
    legal_moves(state)
        .into_iter()
        .filter_map(|m| priority(&m).map(|p| (p, m)))
        .min_by_key(|(p, _)| *p)
        .map(|(_, m)| m)
}
