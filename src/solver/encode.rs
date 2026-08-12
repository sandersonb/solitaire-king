//! Compact, canonical byte encoding of a game position — the cycle-detection key.
//!
//! Layout (all lengths fit in a byte for a 52-card game):
//! - 7 tableau columns, each `[len, card…]` in column order (face-up flag kept);
//! - stock `[len, card…]`, waste `[len, card…]`;
//! - 4 foundation bytes: the top rank present per suit in `Suit::ALL` order
//!   (0 = empty) — canonical, so foundation-slot permutations encode identically;
//! - one trailing byte with recycles remaining, **only** when the redeal limit
//!   is bounded (it never changes legality under unlimited redeals).

use crate::{Card, GameState, Suit};

/// A compact, hashable encoding of a position. Cheap to clone and compare.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PositionKey(Vec<u8>);

impl PositionKey {
    /// Number of bytes in the encoding.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Encode a single card into one byte: rank (bits 0-3), suit (bits 4-5),
/// face-up (bit 6).
fn encode_card(card: Card) -> u8 {
    card.rank.value() | ((card.suit as u8) << 4) | ((card.face_up as u8) << 6)
}

/// Encode a full position into a canonical [`PositionKey`].
pub fn encode(state: &GameState) -> PositionKey {
    let mut bytes = Vec::with_capacity(80);

    // Tableau columns in order, length-prefixed, with face-up flags.
    for col in state.tableau.iter() {
        let cards = col.cards();
        bytes.push(cards.len() as u8);
        bytes.extend(cards.iter().map(|c| encode_card(*c)));
    }

    // Stock and waste, length-prefixed, in order.
    for pile in [state.stock.cards(), state.waste.cards()] {
        bytes.push(pile.len() as u8);
        bytes.extend(pile.iter().map(|c| encode_card(*c)));
    }

    // Foundations, canonical by suit: top rank present per suit (0 if empty).
    for suit in Suit::ALL {
        let top_rank = state
            .foundations
            .iter()
            .find(|f| f.suit() == Some(suit))
            .and_then(|f| f.top())
            .map(|c| c.rank.value())
            .unwrap_or(0);
        bytes.push(top_rank);
    }

    // Recycles remaining — only when bounded (affects legality); omit if unlimited.
    if let Some(limit) = state.config().redeal_limit {
        let remaining = limit.saturating_sub(state.recycles_done());
        bytes.push(remaining.min(u8::MAX as u32) as u8);
    }

    PositionKey(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DrawMode, GameConfig};

    #[test]
    fn deterministic() {
        let g = GameState::new_with_seed(42, GameConfig::default());
        assert_eq!(encode(&g), encode(&g));
    }

    #[test]
    fn distinct_positions_differ() {
        let mut g = GameState::new_with_seed(42, GameConfig::default());
        let before = encode(&g);
        g.apply(crate::Move::Draw).unwrap();
        assert_ne!(before, encode(&g), "drawing must change the encoding");
    }

    #[test]
    fn excludes_score_and_time() {
        let a = GameState::new_with_seed(7, GameConfig::default());
        let mut b = a.clone();
        b.set_elapsed_secs(9999); // affects score/time, not the position
        assert_eq!(encode(&a), encode(&b));
    }

    #[test]
    fn unlimited_redeal_ignores_recycle_count() {
        // Under the default (unlimited) redeal, recycle count is not encoded, so
        // the byte length is fixed regardless of recycles performed.
        let g = GameState::new_with_seed(1, GameConfig::default());
        assert!(g.config().redeal_limit.is_none());
        let len_unlimited = encode(&g).len();

        let bounded = GameConfig {
            draw_mode: DrawMode::Three,
            redeal_limit: Some(3),
            timed: false,
        };
        let gb = GameState::new_with_seed(1, bounded);
        // Bounded redeal appends one extra byte (recycles remaining).
        assert_eq!(encode(&gb).len(), len_unlimited + 1);
    }
}
