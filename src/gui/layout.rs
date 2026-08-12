//! Responsive board layout: turns the window size into pile rectangles and
//! provides hit-test geometry.

use macroquad::prelude::Rect;

pub const NUM_TABLEAU: usize = 7;

/// Card height as a multiple of card width.
const CARD_ASPECT: f32 = 1.4;
/// Minimum vertical fan spacing between stacked cards, as a fraction of card height.
const MIN_FAN_FRAC: f32 = 0.10;

pub struct Layout {
    pub card_w: f32,
    pub stock: Rect,
    pub waste: Rect,
    pub foundations: [Rect; 4],
    /// Top-card rect of each tableau column.
    pub tableau: [Rect; NUM_TABLEAU],
    /// Vertical offset between successive cards in a column.
    pub fan_dy: f32,
}

impl Layout {
    /// Build the layout for the window size, sizing the tableau fan so the
    /// tallest column (`max_col_len` cards) fits the available height.
    pub fn compute(sw: f32, sh: f32, max_col_len: usize) -> Layout {
        let margin = sw * 0.02;
        let gap = margin * 0.6;
        // Width budget: seven columns span the width, with gaps between.
        let width_card_w =
            ((sw - 2.0 * margin) - gap * (NUM_TABLEAU as f32 - 1.0)) / NUM_TABLEAU as f32;

        // Height budget: also cap the card size so a wide, short viewport (e.g. a
        // maximized browser) can't blow the cards up and clip the tallest column.
        // Vertically we must fit: top strip + top-row card + gap + the tallest
        // column at minimum fan spacing (card_h * MIN_FAN_FRAC per extra card).
        let n = max_col_len.max(1) as f32;
        let col_span = 2.0 + MIN_FAN_FRAC * (n - 1.0); // top-row card + tallest column
        let height_card_h = ((sh * 0.94 - 3.0 * margin) / col_span).max(1.0);
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

        // Compress the fan so the tallest column fits between `tab_y` and the
        // bottom; keep a comfortable spread when columns are short.
        let default_fan = card_h * 0.28;
        let avail = (sh - tab_y - margin).max(card_h);
        let fan_dy = if max_col_len > 1 {
            ((avail - card_h) / (max_col_len as f32 - 1.0))
                .clamp(card_h * MIN_FAN_FRAC, default_fan)
        } else {
            default_fan
        };

        Layout {
            card_w,
            stock,
            waste,
            foundations,
            tableau,
            fan_dy,
        }
    }

    /// Rect of the card at `index` (0 = bottom) in tableau column `col`.
    pub fn tableau_card_rect(&self, col: usize, index: usize) -> Rect {
        let base = self.tableau[col];
        Rect::new(base.x, base.y + index as f32 * self.fan_dy, base.w, base.h)
    }
}
