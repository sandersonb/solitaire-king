//! Cards: suits, ranks, colors, and the `Card` value type.

use std::fmt;

/// The color of a card, derived from its suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Black,
}

/// The four suits of a standard deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    /// All four suits, in a stable order.
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    /// The color of this suit. Hearts and Diamonds are red; Clubs and Spades are black.
    pub fn color(self) -> Color {
        match self {
            Suit::Hearts | Suit::Diamonds => Color::Red,
            Suit::Clubs | Suit::Spades => Color::Black,
        }
    }

    /// A single-character symbol for display (♣ ♦ ♥ ♠).
    pub fn symbol(self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }
}

/// The thirteen ranks. The backing values run 1..=13 so that sequencing is
/// plain integer arithmetic: Ace = 1, King = 13.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

impl Rank {
    /// All thirteen ranks in ascending order (Ace..King).
    pub const ALL: [Rank; 13] = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];

    /// The integer value of this rank (Ace = 1 … King = 13).
    pub fn value(self) -> u8 {
        self as u8
    }

    /// The rank exactly one higher, or `None` for King.
    pub fn succ(self) -> Option<Rank> {
        Rank::ALL.get(self.value() as usize).copied()
    }

    /// A short label for display (A, 2..10, J, Q, K).
    pub fn label(self) -> &'static str {
        match self {
            Rank::Ace => "A",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        }
    }
}

/// A single playing card. Small and `Copy` so pile manipulation is cheap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
    /// Whether the card is currently face-up (visible) to the player.
    pub face_up: bool,
}

impl Card {
    /// Create a face-down card of the given rank and suit.
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card {
            rank,
            suit,
            face_up: false,
        }
    }

    /// The color of this card, derived from its suit.
    pub fn color(self) -> Color {
        self.suit.color()
    }

    /// Return a copy of this card turned face-up.
    pub fn face_up(self) -> Self {
        Card {
            face_up: true,
            ..self
        }
    }

    /// Return a copy of this card turned face-down.
    pub fn face_down(self) -> Self {
        Card {
            face_up: false,
            ..self
        }
    }

    /// Turn this card face-up in place.
    pub fn flip_up(&mut self) {
        self.face_up = true;
    }

    /// Turn this card face-down in place.
    pub fn flip_down(&mut self) {
        self.face_up = false;
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.face_up {
            write!(f, "{}{}", self.rank.label(), self.suit.symbol())
        } else {
            write!(f, "[]")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_suit() {
        assert_eq!(Suit::Hearts.color(), Color::Red);
        assert_eq!(Suit::Diamonds.color(), Color::Red);
        assert_eq!(Suit::Clubs.color(), Color::Black);
        assert_eq!(Suit::Spades.color(), Color::Black);
    }

    #[test]
    fn rank_values_and_ordering() {
        assert_eq!(Rank::Ace.value(), 1);
        assert_eq!(Rank::King.value(), 13);
        assert!(Rank::Ace < Rank::King);
        // Every rank has a distinct value in 1..=13.
        let mut seen = std::collections::HashSet::new();
        for r in Rank::ALL {
            assert!((1..=13).contains(&r.value()));
            assert!(seen.insert(r.value()), "duplicate rank value");
        }
    }

    #[test]
    fn rank_succ() {
        assert_eq!(Rank::Ace.succ(), Some(Rank::Two));
        assert_eq!(Rank::Ten.succ(), Some(Rank::Jack));
        assert_eq!(Rank::King.succ(), None);
    }

    #[test]
    fn flip_helpers() {
        let c = Card::new(Rank::Ace, Suit::Spades);
        assert!(!c.face_up);
        assert!(c.face_up().face_up);
        let mut m = c;
        m.flip_up();
        assert!(m.face_up);
        m.flip_down();
        assert!(!m.face_up);
    }
}
