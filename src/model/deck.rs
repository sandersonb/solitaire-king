//! The standard 52-card deck and deterministic shuffling.

use crate::model::card::{Card, Rank, Suit};
use crate::model::rng::SplitMix64;

/// A deck of cards. Order is significant; index 0 is the "bottom".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    /// Construct a standard, ordered 52-card deck: every (rank, suit)
    /// combination exactly once, all face-down. No jokers.
    pub fn standard() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                cards.push(Card::new(rank, suit));
            }
        }
        Deck { cards }
    }

    /// Number of cards in the deck.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Whether the deck is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Shuffle the deck in place using an unbiased Fisher–Yates driven by the
    /// given deterministic PRNG. The same generator state yields the same order.
    pub fn shuffle(&mut self, rng: &mut SplitMix64) {
        let n = self.cards.len();
        if n <= 1 {
            return;
        }
        // Fisher–Yates: for i from n-1 down to 1, swap i with a uniform j in 0..=i.
        for i in (1..n).rev() {
            let j = rng.next_below((i + 1) as u64) as usize;
            self.cards.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn standard_deck_has_52_unique_cards() {
        let deck = Deck::standard();
        assert_eq!(deck.len(), 52);
        let mut seen = HashSet::new();
        for c in &deck.cards {
            assert!(seen.insert((c.rank, c.suit)), "duplicate card {c:?}");
        }
        assert_eq!(seen.len(), 52);
    }

    #[test]
    fn shuffle_is_deterministic() {
        let mut a = Deck::standard();
        let mut b = Deck::standard();
        a.shuffle(&mut SplitMix64::new(12345));
        b.shuffle(&mut SplitMix64::new(12345));
        assert_eq!(a, b);
    }

    #[test]
    fn shuffle_preserves_multiset() {
        let original: HashSet<_> = Deck::standard()
            .cards
            .iter()
            .map(|c| (c.rank, c.suit))
            .collect();
        let mut deck = Deck::standard();
        deck.shuffle(&mut SplitMix64::new(99));
        let shuffled: HashSet<_> = deck.cards.iter().map(|c| (c.rank, c.suit)).collect();
        assert_eq!(original, shuffled);
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn different_seeds_usually_differ() {
        let mut a = Deck::standard();
        let mut b = Deck::standard();
        a.shuffle(&mut SplitMix64::new(1));
        b.shuffle(&mut SplitMix64::new(2));
        assert_ne!(a, b);
    }
}
