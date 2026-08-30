//! Responsive board layout: turns the window size into pile rectangles and
//! provides hit-test and drop-zone geometry, plus an on-screen control bar so
//! the game is playable by touch alone.

use macroquad::prelude::Rect;

pub const NUM_TABLEAU: usize = 7;

/// Card height as a multiple of card width.
const CARD_ASPECT: f32 = 1.4;
/// Minimum vertical fan spacing between stacked cards, as a fraction of card height.
const MIN_FAN_FRAC: f32 = 0.10;

/// An on-screen control-bar button.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonId {
    Undo,
    New,
}

pub struct Layout {
    pub card_w: f32,
    pub stock: Rect,
    pub waste: Rect,
    pub foundations: [Rect; 4],
    /// Top-card rect of each tableau column.
    pub tableau: [Rect; NUM_TABLEAU],
    /// Vertical offset between successive cards in a column.
    pub fan_dy: f32,
    /// True on narrow/portrait/touch viewports (prefers mobile card art, etc.).
    pub mobile: bool,
    /// The on-screen control bar along the bottom edge.
    pub bar: Rect,
    /// On-screen buttons within the bar.
    pub buttons: [(ButtonId, Rect); 2],
}

impl Layout {
    /// Build the layout for the window size, sizing the tableau fan so the
    /// tallest column (`max_col_len` cards) fits the available height. `touch`
    /// forces the mobile profile even on a wide viewport.
    pub fn compute(sw: f32, sh: f32, max_col_len: usize, touch: bool) -> Layout {
        // Mobile profile: touch device, portrait, or a narrow window.
        let mobile = touch || sw < sh || sw < 700.0;

        let margin = sw * 0.02;
        let gap = margin * 0.6;

        // Reserve a control-bar band along the bottom (taller touch targets on
        // mobile). The board must not draw under it.
        let bar_h = (sh * if mobile { 0.09 } else { 0.06 }).clamp(40.0, 88.0);
        let bar = Rect::new(0.0, sh - bar_h, sw, bar_h);

        // Width budget: seven columns span the width, with gaps between.
        let width_card_w =
            ((sw - 2.0 * margin) - gap * (NUM_TABLEAU as f32 - 1.0)) / NUM_TABLEAU as f32;

        // Height budget: also cap the card size so a wide, short viewport (e.g. a
        // maximized browser) can't blow the cards up and clip the tallest column.
        // The usable height excludes the control bar.
        let usable_h = sh - bar_h;
        let n = max_col_len.max(1) as f32;
        let col_span = 2.0 + MIN_FAN_FRAC * (n - 1.0); // top-row card + tallest column
        let height_card_h = ((usable_h * 0.96 - 3.0 * margin) / col_span).max(1.0);
        let height_card_w = height_card_h / CARD_ASPECT;

        let card_w = width_card_w.min(height_card_w);
        let card_h = card_w * CARD_ASPECT;

        // Center horizontally when width isn't the binding constraint.
        let board_w = NUM_TABLEAU as f32 * card_w + (NUM_TABLEAU as f32 - 1.0) * gap;
        let offset_x = ((sw - board_w) / 2.0).max(margin);
        let col_x = |i: usize| offset_x + i as f32 * (card_w + gap);

        let top_y = sh * 0.06 + margin;
        let stock = Rect::new(col_x(0), top_y, card_w, card_h);
        let waste = Rect::new(col_x(1), top_y, card_w, card_h);
        let foundations = [
            Rect::new(col_x(3), top_y, card_w, card_h),
            Rect::new(col_x(4), top_y, card_w, card_h),
            Rect::new(col_x(5), top_y, card_w, card_h),
            Rect::new(col_x(6), top_y, card_w, card_h),
        ];

        let tab_y = top_y + card_h + margin;
        let mut tableau = [Rect::new(0.0, 0.0, card_w, card_h); NUM_TABLEAU];
        for (i, r) in tableau.iter_mut().enumerate() {
            *r = Rect::new(col_x(i), tab_y, card_w, card_h);
        }

        // Compress the fan so the tallest column fits between `tab_y` and the top
        // of the control bar; keep a comfortable spread when columns are short.
        let default_fan = card_h * 0.28;
        let avail = (bar.y - tab_y - margin).max(card_h);
        let fan_dy = if max_col_len > 1 {
            ((avail - card_h) / (max_col_len as f32 - 1.0))
                .clamp(card_h * MIN_FAN_FRAC, default_fan)
        } else {
            default_fan
        };

        // Two buttons on the right of the bar (Undo, New), as touch targets.
        let bh = bar_h * 0.72;
        let bw = (sw * 0.20).clamp(90.0, 190.0);
        let pad = (bar_h - bh) / 2.0;
        let by = bar.y + pad;
        let new_x = sw - margin - bw;
        let undo_x = new_x - gap - bw;
        let buttons = [
            (ButtonId::Undo, Rect::new(undo_x, by, bw, bh)),
            (ButtonId::New, Rect::new(new_x, by, bw, bh)),
        ];

        Layout {
            card_w,
            stock,
            waste,
            foundations,
            tableau,
            fan_dy,
            mobile,
            bar,
            buttons,
        }
    }

    /// Rect of the card at `index` (0 = bottom) in tableau column `col`.
    pub fn tableau_card_rect(&self, col: usize, index: usize) -> Rect {
        let base = self.tableau[col];
        Rect::new(base.x, base.y + index as f32 * self.fan_dy, base.w, base.h)
    }

    /// The button at point `(x, y)`, if any.
    pub fn button_at(&self, x: f32, y: f32) -> Option<ButtonId> {
        self.buttons
            .iter()
            .find(|(_, r)| contains(*r, x, y))
            .map(|(id, _)| *id)
    }
}

fn contains(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h
}
