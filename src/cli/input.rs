//! Key → pile mapping and pure move-resolution helpers.
//!
//! Everything here is pure: it reads a `GameState` (via `legal_moves`) and
//! returns candidate `Move`s, so it is unit-testable without a terminal. The
//! rules engine remains the single source of truth for legality — these helpers
//! only ever return moves drawn from `legal_moves`.

use crate::cli::Pile;
use klondike::{legal_moves, GameState, Move};

/// Map a character to the pile it addresses, if any.
///
/// `1`–`7` → tableau columns 0–6; `8 9 0 -` → foundations 0–3; space → the
/// stock/waste corner.
pub fn key_to_pile(c: char) -> Option<Pile> {
    match c {
        '1'..='7' => Some(Pile::Tableau(c as usize - '1' as usize)),
        '8' => Some(Pile::Foundation(0)),
        '9' => Some(Pile::Foundation(1)),
        '0' => Some(Pile::Foundation(2)),
        '-' => Some(Pile::Foundation(3)),
        ' ' => Some(Pile::StockWaste),
        _ => None,
    }
}

/// Resolve a source→target selection into a concrete legal move, if one exists.
///
/// Handles forgiving foundation targeting (any foundation key routes to the
/// suit's foundation) by matching against `legal_moves`. For tableau→tableau it
/// returns the move only when exactly one run length is legal; when several are
/// legal it returns `Err(RunAmbiguity)` carrying the choices so the caller can
/// prompt.
pub fn resolve_move(
    state: &GameState,
    source: Pile,
    target: Pile,
) -> Result<Option<Move>, RunAmbiguity> {
    let legal = legal_moves(state);
    match (source, target) {
        // Waste top → foundation (forgiving: any foundation key).
        (Pile::StockWaste, Pile::Foundation(_)) => Ok(legal
            .into_iter()
            .find(|m| matches!(m, Move::WasteToFoundation { .. }))),
        // Waste top → a specific tableau column.
        (Pile::StockWaste, Pile::Tableau(k)) => Ok(legal
            .into_iter()
            .find(|m| matches!(m, Move::WasteToTableau { column } if *column == k))),
        // Tableau top → foundation (forgiving).
        (Pile::Tableau(i), Pile::Foundation(_)) => Ok(legal
            .into_iter()
            .find(|m| matches!(m, Move::TableauToFoundation { column, .. } if *column == i))),
        // Foundation top → a specific tableau column.
        (Pile::Foundation(i), Pile::Tableau(k)) => Ok(legal.into_iter().find(|m| {
            matches!(m, Move::FoundationToTableau { foundation, column }
                if *foundation == i && *column == k)
        })),
        // Tableau run → tableau column: may involve a run-length choice.
        (Pile::Tableau(i), Pile::Tableau(k)) => {
            let mut counts: Vec<usize> = legal
                .iter()
                .filter_map(|m| match m {
                    Move::TableauToTableau { from, to, count } if *from == i && *to == k => {
                        Some(*count)
                    }
                    _ => None,
                })
                .collect();
            counts.sort_unstable();
            match counts.len() {
                0 => Ok(None),
                1 => Ok(Some(Move::TableauToTableau {
                    from: i,
                    to: k,
                    count: counts[0],
                })),
                _ => Err(RunAmbiguity {
                    from: i,
                    to: k,
                    counts,
                }),
            }
        }
        // Anything else (e.g. target is the stock/waste, or a no-op) is illegal.
        _ => Ok(None),
    }
}

/// More than one run length is legal for a tableau→tableau move; the caller
/// should prompt the player to pick a `count`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunAmbiguity {
    pub from: usize,
    pub to: usize,
    pub counts: Vec<usize>,
}

/// Choose the best legal move for `source`, greedily favoring the move that
/// relocates the most cards (largest tableau run first, down to the least);
/// ties are broken by preferring a foundation. Returns `None` if nothing is
/// legal. `source` of `StockWaste` acts on the waste's top card.
pub fn auto_target(state: &GameState, source: Pile) -> Option<Move> {
    legal_moves(state)
        .into_iter()
        .filter(|m| move_matches_source(source, *m))
        .max_by_key(|m| (cards_moved(*m), to_foundation(*m) as u8))
}

/// Whether `m` originates from `source`.
fn move_matches_source(source: Pile, m: Move) -> bool {
    match (source, m) {
        (Pile::StockWaste, Move::WasteToFoundation { .. }) => true,
        (Pile::StockWaste, Move::WasteToTableau { .. }) => true,
        (Pile::Tableau(i), Move::TableauToFoundation { column, .. }) => column == i,
        (Pile::Tableau(i), Move::TableauToTableau { from, .. }) => from == i,
        (Pile::Foundation(i), Move::FoundationToTableau { foundation, .. }) => foundation == i,
        _ => false,
    }
}

/// How many cards a move relocates (tableau runs move `count`; all else move 1).
fn cards_moved(m: Move) -> usize {
    match m {
        Move::TableauToTableau { count, .. } => count,
        _ => 1,
    }
}

/// Whether a move lands on a foundation (used as the auto-assign tiebreaker).
fn to_foundation(m: Move) -> bool {
    matches!(
        m,
        Move::WasteToFoundation { .. } | Move::TableauToFoundation { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use klondike::{Card, Foundation, GameConfig, Rank, Suit, TableauColumn};

    fn up(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit).face_up()
    }

    /// A bare state we can hand-arrange for focused resolution tests.
    fn blank() -> GameState {
        let mut g = GameState::new_with_seed(0, GameConfig::default());
        g.stock = Default::default();
        g.waste = Default::default();
        g.foundations = Default::default();
        g.tableau = Default::default();
        g
    }

    #[test]
    fn foundation_to_tableau_resolves() {
        let mut g = blank();
        let mut f = Foundation::new();
        f.push(up(Rank::Ace, Suit::Hearts));
        f.push(up(Rank::Two, Suit::Hearts));
        g.foundations[0] = f;
        // 3♠ (black) top accepts the red 2♥ from the foundation.
        g.tableau[0] = TableauColumn::new(vec![up(Rank::Three, Suit::Spades)]);
        assert_eq!(
            resolve_move(&g, Pile::Foundation(0), Pile::Tableau(0)),
            Ok(Some(Move::FoundationToTableau {
                foundation: 0,
                column: 0
            }))
        );
    }

    #[test]
    fn auto_assign_prefers_the_largest_run() {
        let mut g = blank();
        // Source column: valid face-up run 9♥ 8♠ 7♦ (red/black/red descending).
        g.tableau[0] = TableauColumn::new(vec![
            up(Rank::Nine, Suit::Hearts),
            up(Rank::Eight, Suit::Spades),
            up(Rank::Seven, Suit::Diamonds),
        ]);
        // 10♠ accepts the full 3-card run (bottom 9♥); 9♦ accepts only the 2-card
        // run (bottom 8♠). Greedy auto should pick the 3-card move.
        g.tableau[1] = TableauColumn::new(vec![up(Rank::Ten, Suit::Spades)]);
        g.tableau[2] = TableauColumn::new(vec![up(Rank::Nine, Suit::Diamonds)]);
        assert_eq!(
            auto_target(&g, Pile::Tableau(0)),
            Some(Move::TableauToTableau {
                from: 0,
                to: 1,
                count: 3
            })
        );
    }

    #[test]
    fn key_mapping() {
        assert_eq!(key_to_pile('1'), Some(Pile::Tableau(0)));
        assert_eq!(key_to_pile('7'), Some(Pile::Tableau(6)));
        assert_eq!(key_to_pile('8'), Some(Pile::Foundation(0)));
        assert_eq!(key_to_pile('9'), Some(Pile::Foundation(1)));
        assert_eq!(key_to_pile('0'), Some(Pile::Foundation(2)));
        assert_eq!(key_to_pile('-'), Some(Pile::Foundation(3)));
        assert_eq!(key_to_pile(' '), Some(Pile::StockWaste));
        assert_eq!(key_to_pile('x'), None);
        assert_eq!(key_to_pile('4'), Some(Pile::Tableau(3)));
    }
}
