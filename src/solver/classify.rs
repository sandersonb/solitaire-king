//! Move classification: the provably-safe no-op reduction and the optional,
//! off-by-default equivalence reduction.

use std::collections::HashSet;

use crate::model::rules::legal_moves;
use crate::{Color, GameState, Move};

/// Whether applying `mv` to `pre` (producing `child`) is a **no-op**: a
/// tableau→tableau move that reveals no face-down card, lands on an
/// interchangeable host, and leaves the exposed source card with no legal move.
/// Such a move only shuffles interchangeable cards among interchangeable slots,
/// so any win reachable through it is reachable without it — safe to skip.
///
/// Draws, recycles, foundation-touching moves, and card-revealing moves are
/// never no-ops (handled by the early return / the reveal check).
pub fn is_no_op(pre: &GameState, mv: Move, child: &GameState) -> bool {
    match no_op_structural(pre, mv) {
        Some(from) => child.tableau[from].is_empty() || !column_has_legal_move(child, from),
        None => false,
    }
}

/// The pre-move (structural) half of the no-op test, for make/unmake callers:
/// returns the source column to re-check *after* applying the move when `mv` is
/// a tableau→tableau move that reveals no face-down card and lands on an
/// interchangeable host; otherwise `None` (never a no-op). The caller completes
/// the test by confirming the exposed source card has no legal move.
pub fn no_op_structural(pre: &GameState, mv: Move) -> Option<usize> {
    let (from, to, count) = match mv {
        Move::TableauToTableau { from, to, count } => (from, to, count),
        _ => return None,
    };

    let cards = pre.tableau[from].cards();
    let n = cards.len();

    // (b) Reveals no face-down card: a card left beneath the moved run must
    // already be face-up (otherwise the move flips it — productive).
    let card_beneath = if n > count {
        Some(cards[n - count - 1])
    } else {
        None
    };
    if let Some(beneath) = card_beneath {
        if !beneath.face_up {
            return None;
        }
    }

    // (c) Interchangeable host: empty→empty (a King-headed run between empty
    // columns), or the run's bottom moves from a face-up host of the same rank
    // and color as the destination's top.
    let host_to = pre.tableau[to].top();
    let interchangeable = match (card_beneath, host_to) {
        (None, None) => true,
        (Some(a), Some(c)) => a.rank == c.rank && a.color() == c.color(),
        _ => false,
    };
    if interchangeable {
        Some(from)
    } else {
        None
    }
}

/// Whether any legal move in `state` originates from tableau column `col`.
pub fn column_has_legal_move(state: &GameState, col: usize) -> bool {
    legal_moves(state).iter().any(|m| match m {
        Move::TableauToFoundation { column, .. } => *column == col,
        Move::TableauToTableau { from, .. } => *from == col,
        _ => false,
    })
}

/// **Experimental, off by default.** Collapse equivalent destinations for the
/// waste's top card: if it has several legal tableau destinations whose hosts
/// are interchangeable (multiple empty columns, or hosts of the same rank and
/// color), keep only the first. Soundness is unproven — validate before relying
/// on it.
pub fn apply_equivalence_pruning(state: &GameState, moves: Vec<Move>) -> Vec<Move> {
    let mut kept_empty = false;
    let mut kept_hosts: HashSet<(u8, Color)> = HashSet::new();
    moves
        .into_iter()
        .filter(|mv| match mv {
            Move::WasteToTableau { column } => match state.tableau[*column].top() {
                None => {
                    if kept_empty {
                        false
                    } else {
                        kept_empty = true;
                        true
                    }
                }
                Some(c) => kept_hosts.insert((c.rank.value(), c.color())),
            },
            _ => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, GameConfig, Rank, Suit, TableauColumn};

    fn up(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit).face_up()
    }
    fn down(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    fn blank() -> GameState {
        let mut g = GameState::new_with_seed(0, GameConfig::default());
        g.stock = Default::default();
        g.waste = Default::default();
        g.foundations = Default::default();
        g.tableau = Default::default();
        g
    }

    /// Apply a move to a clone and return the child (helper mirroring the search).
    fn child_of(pre: &GameState, mv: Move) -> GameState {
        let mut c = pre.clone();
        c.apply(mv).expect("legal move");
        c
    }

    #[test]
    fn king_between_empty_columns_is_no_op() {
        let mut g = blank();
        g.tableau[0] = TableauColumn::new(vec![up(Rank::King, Suit::Spades)]);
        // column 1 is empty
        let mv = Move::TableauToTableau {
            from: 0,
            to: 1,
            count: 1,
        };
        assert!(is_no_op(&g, mv, &child_of(&g, mv)));
    }

    #[test]
    fn lateral_shift_between_equivalent_hosts_is_no_op() {
        let mut g = blank();
        // red 2 on black 3 (3♠); another black 3 (3♣) elsewhere; neither 3 can move.
        g.tableau[0] = TableauColumn::new(vec![
            up(Rank::Three, Suit::Spades),
            up(Rank::Two, Suit::Hearts),
        ]);
        g.tableau[1] = TableauColumn::new(vec![up(Rank::Three, Suit::Clubs)]);
        let mv = Move::TableauToTableau {
            from: 0,
            to: 1,
            count: 1,
        };
        assert!(is_no_op(&g, mv, &child_of(&g, mv)));
    }

    #[test]
    fn revealing_move_is_not_a_no_op() {
        let mut g = blank();
        // Moving the red 2 exposes a FACE-DOWN card -> reveal -> not a no-op.
        g.tableau[0] = TableauColumn::new(vec![
            down(Rank::Nine, Suit::Spades),
            up(Rank::Two, Suit::Hearts),
        ]);
        g.tableau[1] = TableauColumn::new(vec![up(Rank::Three, Suit::Clubs)]);
        let mv = Move::TableauToTableau {
            from: 0,
            to: 1,
            count: 1,
        };
        assert!(!is_no_op(&g, mv, &child_of(&g, mv)));
    }

    #[test]
    fn exposed_card_with_a_move_is_not_a_no_op() {
        let mut g = blank();
        // Move red 2 off black 3 (3♠); 3♠ CAN then go onto a red 4 -> productive.
        g.tableau[0] = TableauColumn::new(vec![
            up(Rank::Three, Suit::Spades),
            up(Rank::Two, Suit::Hearts),
        ]);
        g.tableau[1] = TableauColumn::new(vec![up(Rank::Three, Suit::Clubs)]);
        g.tableau[2] = TableauColumn::new(vec![up(Rank::Four, Suit::Diamonds)]); // red 4 accepts 3♠
        let mv = Move::TableauToTableau {
            from: 0,
            to: 1,
            count: 1,
        };
        assert!(!is_no_op(&g, mv, &child_of(&g, mv)));
    }

    #[test]
    fn stock_and_foundation_moves_are_never_no_ops() {
        let mut g = blank();
        g.tableau[0] = TableauColumn::new(vec![up(Rank::Ace, Suit::Hearts)]);
        // Draw and foundation moves short-circuit to false.
        let child = g.clone();
        assert!(!is_no_op(&g, Move::Draw, &child));
        assert!(!is_no_op(
            &g,
            Move::TableauToFoundation {
                column: 0,
                foundation: 0
            },
            &child
        ));
    }
}
