//! 128-bit Zobrist position hashing.
//!
//! A fixed random value is assigned to each position *feature*; a position's
//! hash is the XOR of its features. The canonicalization matches the byte
//! encoding ([`super::encode`]): positional-only (no score/time), foundations
//! keyed by suit (slots interchangeable), and recycles-remaining included only
//! when the redeal is bounded. The feature table is generated once from the
//! in-crate PRNG with a fixed seed, so hashing is deterministic.

use std::sync::OnceLock;

use crate::model::rng::SplitMix64;
use crate::{Card, GameState, Suit};

const NUM_CARDS: usize = 52;
/// Non-foundation piles: 7 tableau columns, then stock (7), then waste (8).
const NUM_PILES: usize = 9;
const STOCK_PILE: usize = 7;
const WASTE_PILE: usize = 8;
const MAX_DEPTH: usize = 32; // > the deepest possible pile (stock/waste = 24)
const NUM_FACE: usize = 2;
const MAX_RECYCLES: usize = 64;

/// The fixed table of per-feature random values.
struct ZobristTable {
    /// Indexed by (card, pile, depth, face).
    cards: Vec<u128>,
    /// Indexed by (suit, top rank 1..=13).
    foundations: [u128; 4 * 14],
    /// Indexed by recycles-remaining (saturated).
    recycles: [u128; MAX_RECYCLES],
}

fn table() -> &'static ZobristTable {
    static TABLE: OnceLock<ZobristTable> = OnceLock::new();
    TABLE.get_or_init(ZobristTable::generate)
}

impl ZobristTable {
    fn generate() -> Self {
        let mut rng = SplitMix64::new(0x5A0B_1257_ABCD_1234);
        let mut next = || {
            let lo = rng.next_u64() as u128;
            let hi = rng.next_u64() as u128;
            (hi << 64) | lo
        };
        let cards = (0..NUM_CARDS * NUM_PILES * MAX_DEPTH * NUM_FACE)
            .map(|_| next())
            .collect();
        let mut foundations = [0u128; 4 * 14];
        for slot in foundations.iter_mut() {
            *slot = next();
        }
        let mut recycles = [0u128; MAX_RECYCLES];
        for slot in recycles.iter_mut() {
            *slot = next();
        }
        ZobristTable {
            cards,
            foundations,
            recycles,
        }
    }

    fn card(&self, card: Card, pile: usize, depth: usize) -> u128 {
        let cid = (card.suit as usize) * 13 + (card.rank.value() as usize - 1);
        let d = depth.min(MAX_DEPTH - 1);
        let idx = (((cid * NUM_PILES + pile) * MAX_DEPTH + d) * NUM_FACE) + card.face_up as usize;
        self.cards[idx]
    }

    fn foundation(&self, suit: Suit, rank: u8) -> u128 {
        self.foundations[(suit as usize) * 14 + rank as usize]
    }

    fn recycles(&self, remaining: u32) -> u128 {
        self.recycles[(remaining as usize).min(MAX_RECYCLES - 1)]
    }
}

/// The 128-bit Zobrist hash of `state`'s position.
pub fn zobrist(state: &GameState) -> u128 {
    let t = table();
    let mut h: u128 = 0;

    for (pile, col) in state.tableau.iter().enumerate() {
        for (depth, card) in col.cards().iter().enumerate() {
            h ^= t.card(*card, pile, depth);
        }
    }
    for (depth, card) in state.stock.cards().iter().enumerate() {
        h ^= t.card(*card, STOCK_PILE, depth);
    }
    for (depth, card) in state.waste.cards().iter().enumerate() {
        h ^= t.card(*card, WASTE_PILE, depth);
    }

    // Foundations, canonical by suit (cards on foundations are counted only here).
    for suit in Suit::ALL {
        let rank = state
            .foundations
            .iter()
            .find(|f| f.suit() == Some(suit))
            .and_then(|f| f.top())
            .map_or(0, |c| c.rank.value());
        if rank > 0 {
            h ^= t.foundation(suit, rank);
        }
    }

    if let Some(limit) = state.config().redeal_limit {
        h ^= t.recycles(limit.saturating_sub(state.recycles_done()));
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::rules::legal_moves;
    use crate::solver::encode::encode;
    use crate::{DrawMode, GameConfig};

    #[test]
    fn deterministic_and_positional_only() {
        let a = GameState::new_with_seed(7, GameConfig::default());
        assert_eq!(zobrist(&a), zobrist(&a));
        let mut b = a.clone();
        b.set_elapsed_secs(9999); // score/time must not affect the hash
        assert_eq!(zobrist(&a), zobrist(&b));
    }

    #[test]
    fn foundation_slots_are_interchangeable() {
        use crate::{Card, Foundation, Rank};
        let mut a = GameState::new_with_seed(0, GameConfig::default());
        a.stock = Default::default();
        a.waste = Default::default();
        a.foundations = Default::default();
        a.tableau = Default::default();
        let mut b = a.clone();
        let ace_h = Card::new(Rank::Ace, Suit::Hearts).face_up();
        let ace_s = Card::new(Rank::Ace, Suit::Spades).face_up();
        // Same contents, different foundation slots.
        a.foundations[0] = {
            let mut f = Foundation::new();
            f.push(ace_h);
            f
        };
        a.foundations[1] = {
            let mut f = Foundation::new();
            f.push(ace_s);
            f
        };
        b.foundations[2] = {
            let mut f = Foundation::new();
            f.push(ace_h);
            f
        };
        b.foundations[3] = {
            let mut f = Foundation::new();
            f.push(ace_s);
            f
        };
        assert_eq!(zobrist(&a), zobrist(&b));
    }

    #[test]
    fn hash_identity_matches_byte_encoding() {
        // Over a walk, collect (byte key, zobrist) and assert they induce the
        // same equality relation: encode_eq(x,y) <=> zobrist_eq(x,y).
        for cfg in [
            GameConfig::default(),
            GameConfig {
                draw_mode: DrawMode::One,
                redeal_limit: Some(2),
                timed: false,
            },
        ] {
            let mut state = GameState::new_with_seed(42, cfg);
            let mut samples = Vec::new();
            for step in 0..120 {
                samples.push((encode(&state), zobrist(&state)));
                let moves = legal_moves(&state);
                if moves.is_empty() {
                    break;
                }
                crate::apply_undoable(&mut state, moves[step % moves.len()]).unwrap();
            }
            for (i, (e1, z1)) in samples.iter().enumerate() {
                for (e2, z2) in samples.iter().skip(i) {
                    assert_eq!(
                        e1 == e2,
                        z1 == z2,
                        "zobrist equality must match byte-encoding equality"
                    );
                }
            }
        }
    }
}
