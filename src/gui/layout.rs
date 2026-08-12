//! Responsive board layout: turns the window size into pile rectangles and
//! provides hit-test geometry.

use macroquad::prelude::Rect;

pub const NUM_TABLEAU: usize = 7;

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
        // Seven columns span the width, with gaps between.
        let card_w = ((sw - 2.0 * margin) - gap * (NUM_TABLEAU as f32 - 1.0)) / NUM_TABLEAU as f32;
        let card_h = card_w * 1.4;
        let col_x = |i: usize| margin + i as f32 * (card_w + gap);

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
            ((avail - card_h) / (max_col_len as f32 - 1.0)).clamp(card_h * 0.10, default_fan)
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
