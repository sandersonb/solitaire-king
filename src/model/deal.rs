//! Dealing a fresh Klondike layout from a seed.
//!
//! The deal is the classic row-by-row method and is fully determined by the
//! seed: shuffle a standard deck with the in-crate PRNG, then deal cards across
//! the seven columns so that column `c` (0-based) receives `c + 1` cards with
//! only its top card face-up. The remaining 24 cards form the face-down stock.

use crate::model::card::Card;
use crate::model::deck::Deck;
use crate::model::pile::{Foundation, Stock, TableauColumn, Waste};
use crate::model::rng::SplitMix64;

/// The piles produced by a deal, before assembly into a `GameState`.
pub struct Dealt {
    pub stock: Stock,
    pub waste: Waste,
    pub foundations: [Foundation; 4],
    pub tableau: [TableauColumn; 7],
}

/// Deal a fresh layout deterministically from `seed`.
pub fn deal(seed: u64) -> Dealt {
    let mut deck = Deck::standard();
    deck.shuffle(&mut SplitMix64::new(seed));

    // Draw from the top (end) of the shuffled deck.
    let draw = |deck: &mut Deck| -> Card { deck.cards.pop().expect("deck underflow") };

    let mut columns: [Vec<Card>; 7] = Default::default();
    // Row-by-row: on row r, place a card on every column c >= r. The card on
    // row c (the last one a column receives) is face-up; the rest are face-down.
    for r in 0..7usize {
        for (c, col) in columns.iter_mut().enumerate().skip(r) {
            let mut card = draw(&mut deck);
            card.face_up = c == r;
            col.push(card);
        }
    }

    let tableau: [TableauColumn; 7] = columns.map(TableauColumn::new);

    // The remaining 24 cards form the stock, all face-down.
    let mut stock_cards = std::mem::take(&mut deck.cards);
    for c in &mut stock_cards {
        c.face_up = false;
    }
    let stock = Stock::new(stock_cards);

    Dealt {
        stock,
        waste: Waste::default(),
        foundations: Default::default(),
        tableau,
    }
}
