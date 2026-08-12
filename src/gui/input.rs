//! Mouse hit-testing and (pure) move resolution over the model.

use macroquad::prelude::Rect;

use crate::layout::{Layout, NUM_TABLEAU};
use klondike::{legal_moves, GameState, Move};

/// A destination pile a move can target.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pile {
    Tableau(usize),
    Foundation(usize),
    Stock,
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

/// The destination pile a hit targets, if any.
pub fn pile_of(hit: Hit) -> Option<Pile> {
    match hit {
        Hit::Foundation(i) => Some(Pile::Foundation(i)),
        Hit::TableauCard { col, .. } | Hit::TableauEmpty(col) => Some(Pile::Tableau(col)),
        Hit::Stock => Some(Pile::Stock),
        Hit::Waste => Some(Pile::Waste),
        Hit::None => None,
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
