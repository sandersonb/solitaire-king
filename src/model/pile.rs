//! The game's piles: stock, waste, foundations, and tableau columns.
//!
//! Ordering convention: for every pile the **last** element of the backing
//! `Vec` is the top of the pile — the actionable card.

use crate::model::card::{Card, Suit};

/// The face-down draw pile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stock {
    cards: Vec<Card>,
}

impl Stock {
    /// Create a stock from cards (which should be face-down).
    pub fn new(cards: Vec<Card>) -> Self {
        Stock { cards }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// The cards, bottom-to-top (top is last).
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Draw the top card off the stock, if any.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Push a card onto the top of the stock.
    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Drain all cards, top-first (used to reverse a recycle during undo).
    pub fn drain_top_first(&mut self) -> Vec<Card> {
        let mut out = std::mem::take(&mut self.cards);
        out.reverse();
        out
    }
}

/// The face-up pile that cards are drawn onto (also called the talon).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Waste {
    cards: Vec<Card>,
}

impl Waste {
    pub fn new(cards: Vec<Card>) -> Self {
        Waste { cards }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// The cards, bottom-to-top (top is last).
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// The top (most recently drawn) card, if any.
    pub fn top(&self) -> Option<Card> {
        self.cards.last().copied()
    }

    /// Push a card onto the top of the waste.
    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Remove and return the top card, if any.
    pub fn take_top(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Drain all cards, top-first (for recycling back into the stock).
    pub fn drain_top_first(&mut self) -> Vec<Card> {
        let mut out = std::mem::take(&mut self.cards);
        out.reverse();
        out
    }
}

/// A foundation pile, built up by suit from Ace to King. The suit is implied by
/// its cards; an empty foundation has no fixed suit yet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Foundation {
    cards: Vec<Card>,
}

impl Foundation {
    pub fn new() -> Self {
        Foundation { cards: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// The top card of the foundation, if any.
    pub fn top(&self) -> Option<Card> {
        self.cards.last().copied()
    }

    /// The suit this foundation is building, if it holds any cards.
    pub fn suit(&self) -> Option<Suit> {
        self.cards.first().map(|c| c.suit)
    }

    /// Push a card onto the foundation. Callers must validate legality first.
    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Remove and return the top card, if any.
    pub fn take_top(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Whether the foundation is complete (Ace through King, 13 cards).
    pub fn is_complete(&self) -> bool {
        self.cards.len() == 13
    }
}

/// A tableau column. Invariant: any face-down cards form a contiguous prefix at
/// the bottom, and all face-up cards sit above them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableauColumn {
    cards: Vec<Card>,
}

impl TableauColumn {
    pub fn new(cards: Vec<Card>) -> Self {
        let col = TableauColumn { cards };
        debug_assert!(
            col.invariant_holds(),
            "tableau face-up-prefix invariant violated"
        );
        col
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// The top (last) card of the column, if any.
    pub fn top(&self) -> Option<Card> {
        self.cards.last().copied()
    }

    /// Index of the first face-up card, i.e. the length of the face-down prefix.
    /// Equals `len()` when there are no face-up cards.
    pub fn first_face_up_index(&self) -> usize {
        self.cards
            .iter()
            .position(|c| c.face_up)
            .unwrap_or(self.cards.len())
    }

    /// The contiguous face-up run at the top of the column, bottom-to-top.
    pub fn face_up_run(&self) -> &[Card] {
        &self.cards[self.first_face_up_index()..]
    }

    /// Push a card onto the top of the column.
    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
        debug_assert!(self.invariant_holds());
    }

    /// Remove and return the top card, if any.
    pub fn take_top(&mut self) -> Option<Card> {
        let c = self.cards.pop();
        debug_assert!(self.invariant_holds());
        c
    }

    /// Split off the top `count` cards as an ordered run (bottom-to-top),
    /// leaving the rest in the column. Returns `None` if `count` exceeds the
    /// number of face-up cards available.
    pub fn take_run(&mut self, count: usize) -> Option<Vec<Card>> {
        if count == 0 || count > self.face_up_run().len() {
            return None;
        }
        let at = self.cards.len() - count;
        let run = self.cards.split_off(at);
        debug_assert!(self.invariant_holds());
        Some(run)
    }

    /// Append an ordered run (bottom-to-top) onto the top of the column.
    pub fn push_run(&mut self, run: impl IntoIterator<Item = Card>) {
        self.cards.extend(run);
        debug_assert!(self.invariant_holds());
    }

    /// Turn the top card face-down (used to reverse an auto-flip during undo).
    pub fn flip_top_down(&mut self) {
        if let Some(last) = self.cards.last_mut() {
            last.face_up = false;
        }
    }

    /// If the top card is face-down, turn it face-up and report whether a flip
    /// happened. Used by the auto-flip rule after a move exposes a card.
    pub fn flip_top_if_face_down(&mut self) -> bool {
        if let Some(last) = self.cards.last_mut() {
            if !last.face_up {
                last.face_up = true;
                return true;
            }
        }
        false
    }

    /// Check the face-up-prefix invariant: no face-down card sits above a
    /// face-up card.
    pub fn invariant_holds(&self) -> bool {
        let mut seen_face_up = false;
        for c in &self.cards {
            if c.face_up {
                seen_face_up = true;
            } else if seen_face_up {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::card::{Card, Rank, Suit};

    fn up(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit).face_up()
    }
    fn down(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn tableau_invariant_detects_violation() {
        let ok = TableauColumn::new(vec![
            down(Rank::Five, Suit::Clubs),
            up(Rank::Four, Suit::Hearts),
        ]);
        assert!(ok.invariant_holds());

        // A raw column with a face-down card above a face-up one violates it.
        let bad = TableauColumn {
            cards: vec![up(Rank::Four, Suit::Hearts), down(Rank::Five, Suit::Clubs)],
        };
        assert!(!bad.invariant_holds());
    }

    #[test]
    fn flip_top_when_face_down() {
        let mut col = TableauColumn::new(vec![down(Rank::Nine, Suit::Spades)]);
        assert!(col.flip_top_if_face_down());
        assert!(col.top().unwrap().face_up);
        // Already up: no second flip.
        assert!(!col.flip_top_if_face_down());
    }

    #[test]
    fn take_and_push_run() {
        let mut col = TableauColumn::new(vec![
            down(Rank::King, Suit::Clubs),
            up(Rank::Seven, Suit::Hearts),
            up(Rank::Six, Suit::Spades),
        ]);
        // Only two face-up cards; cannot take three.
        assert!(col.take_run(3).is_none());
        let run = col.take_run(2).unwrap();
        assert_eq!(run.len(), 2);
        assert_eq!(col.len(), 1);
        let mut dest = TableauColumn::new(vec![up(Rank::Eight, Suit::Clubs)]);
        dest.push_run(run);
        assert_eq!(dest.len(), 3);
        assert_eq!(dest.top().unwrap().rank, Rank::Six);
    }

    #[test]
    fn foundation_completion_and_suit() {
        let mut f = Foundation::new();
        assert_eq!(f.suit(), None);
        f.push(up(Rank::Ace, Suit::Diamonds));
        assert_eq!(f.suit(), Some(Suit::Diamonds));
        assert!(!f.is_complete());
    }

    #[test]
    fn waste_drain_is_top_first() {
        let mut w = Waste::new(vec![
            up(Rank::Ace, Suit::Clubs),
            up(Rank::Two, Suit::Clubs),
            up(Rank::Three, Suit::Clubs),
        ]);
        // Top is Three; draining top-first yields 3,2,A.
        let drained = w.drain_top_first();
        assert_eq!(drained[0].rank, Rank::Three);
        assert_eq!(drained[2].rank, Rank::Ace);
        assert!(w.is_empty());
    }
}
