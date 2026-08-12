//! Search heuristics: provably-safe forced foundation auto-moves, move
//! ordering, and empty-column symmetry pruning.

use crate::{Color, GameState, Move, Suit};

/// The two suits of the opposite color.
fn opposite_color_suits(color: Color) -> [Suit; 2] {
    match color {
        Color::Red => [Suit::Clubs, Suit::Spades],
        Color::Black => [Suit::Hearts, Suit::Diamonds],
    }
}

/// The top rank on `suit`'s foundation, or 0 if empty.
fn foundation_rank(state: &GameState, suit: Suit) -> u8 {
    state
        .foundations
        .iter()
        .find(|f| f.suit() == Some(suit))
        .and_then(|f| f.top())
        .map_or(0, |c| c.rank.value())
}

/// Whether the card is **safe** to send to its foundation: rank ≤ 2, or both
/// opposite-color foundations are at rank ≥ (card rank − 1). A safe card can
/// never again be needed as a tableau host, so playing it up loses no win.
fn is_safe(state: &GameState, card: crate::Card) -> bool {
    let r = card.rank.value();
    if r <= 2 {
        return true;
    }
    let opp = opposite_color_suits(card.color());
    let min_opp = opp
        .iter()
        .map(|s| foundation_rank(state, *s))
        .min()
        .unwrap_or(0);
    min_opp + 1 >= r
}

/// A safe foundation move among `moves` to force at this node, if any exists.
/// Takes the already-computed legal moves so the caller needn't recompute them.
pub fn safe_move_in(state: &GameState, moves: &[Move]) -> Option<Move> {
    moves.iter().copied().find(|mv| {
        let card = match mv {
            Move::WasteToFoundation { .. } => state.waste.top(),
            Move::TableauToFoundation { column, .. } => state.tableau[*column].top(),
            _ => None,
        };
        card.is_some_and(|c| is_safe(state, c))
    })
}

/// Whether `mv` exposes (reveals) a face-down tableau card.
fn reveals_face_down(state: &GameState, mv: Move) -> bool {
    let (col, count) = match mv {
        Move::TableauToTableau { from, count, .. } => (from, count),
        Move::TableauToFoundation { column, .. } => (column, 1),
        _ => return false,
    };
    let cards = state.tableau[col].cards();
    let n = cards.len();
    n > count && !cards[n - count - 1].face_up
}

/// The source tableau column length for a move (0 if not tableau-sourced).
fn source_len(state: &GameState, mv: Move) -> usize {
    match mv {
        Move::TableauToTableau { from, .. } => state.tableau[from].len(),
        Move::TableauToFoundation { column, .. } => state.tableau[column].len(),
        _ => 0,
    }
}

/// The ordering key for a candidate move (lower sorts first). `prev` is the move
/// that produced the current state (for the consecutive-draw penalty).
pub fn move_priority(
    state: &GameState,
    mv: Move,
    prev: Option<Move>,
    dig_larger_first: bool,
) -> (u8, i32) {
    if reveals_face_down(state, mv) {
        let len = source_len(state, mv) as i32;
        let tie = if dig_larger_first { -len } else { len };
        return (0, tie);
    }
    match mv {
        Move::FoundationToTableau { .. } => (4, 0),
        Move::Draw | Move::Recycle => {
            let prev_was_draw = matches!(prev, Some(Move::Draw) | Some(Move::Recycle));
            (if prev_was_draw { 3 } else { 2 }, 0)
        }
        _ => (1, 0), // productive builds, waste plays, unsafe foundation plays
    }
}

/// Drop symmetric King-to-empty moves: when ≥2 columns are empty, keep only the
/// move targeting the first empty column (empty columns are interchangeable).
pub fn apply_empty_column_symmetry(state: &GameState, moves: Vec<Move>) -> Vec<Move> {
    let empties: Vec<usize> = (0..7).filter(|&c| state.tableau[c].is_empty()).collect();
    if empties.len() < 2 {
        return moves;
    }
    let keep = empties[0];
    moves
        .into_iter()
        .filter(|mv| {
            let dest = match mv {
                Move::WasteToTableau { column } => Some(*column),
                Move::TableauToTableau { to, .. } => Some(*to),
                _ => None,
            };
            // Drop a move targeting a redundant (non-`keep`) empty column.
            !matches!(dest, Some(c) if state.tableau[c].is_empty() && c != keep)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::rules::legal_moves;
    use crate::{Card, Foundation, GameConfig, Rank, TableauColumn};

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

    #[test]
    fn aces_and_twos_always_safe() {
        let g = blank();
        assert!(is_safe(&g, up(Rank::Ace, Suit::Spades)));
        assert!(is_safe(&g, up(Rank::Two, Suit::Hearts)));
    }

    #[test]
    fn higher_card_safe_when_opposite_colors_caught_up_even_if_same_color_lags() {
        let mut g = blank();
        // Blacks (spades, clubs) at 5; reds uneven (diamonds low). A red 6 is safe
        // because both black 5s are already home.
        let mut s = Foundation::new();
        for r in Rank::ALL.into_iter().take(5) {
            s.push(up(r, Suit::Spades));
        }
        let mut c = Foundation::new();
        for r in Rank::ALL.into_iter().take(5) {
            c.push(up(r, Suit::Clubs));
        }
        g.foundations[0] = s;
        g.foundations[1] = c;
        assert!(is_safe(&g, up(Rank::Six, Suit::Hearts)));
        // A black 6 would need reds at 5 — not the case — so it is unsafe.
        assert!(!is_safe(&g, up(Rank::Six, Suit::Spades)));
    }

    #[test]
    fn safe_move_prefers_a_safe_card() {
        let mut g = blank();
        // Ace on a tableau column can be safely auto-played.
        g.tableau[0] = TableauColumn::new(vec![up(Rank::Ace, Suit::Hearts)]);
        assert_eq!(
            safe_move_in(&g, &legal_moves(&g)),
            Some(Move::TableauToFoundation {
                column: 0,
                foundation: 0
            })
        );
    }

    #[test]
    fn reveal_beats_draw_in_ordering() {
        let mut g = blank();
        // A revealing tableau move vs a draw.
        g.tableau[0] = TableauColumn::new(vec![
            down(Rank::Nine, Suit::Spades),
            up(Rank::Ace, Suit::Hearts),
        ]);
        let reveal = Move::TableauToFoundation {
            column: 0,
            foundation: 0,
        };
        let draw = Move::Draw;
        let pr = move_priority(&g, reveal, None, true);
        let pd = move_priority(&g, draw, None, true);
        assert!(pr < pd, "revealing move should sort before a draw");
    }

    #[test]
    fn consecutive_draw_is_deprioritized() {
        let g = blank();
        let fresh = move_priority(&g, Move::Draw, None, true);
        let after_draw = move_priority(&g, Move::Draw, Some(Move::Draw), true);
        assert!(after_draw > fresh);
    }

    #[test]
    fn foundation_to_tableau_sorts_last() {
        let g = blank();
        let ft = move_priority(
            &g,
            Move::FoundationToTableau {
                foundation: 0,
                column: 0,
            },
            None,
            true,
        );
        let draw = move_priority(&g, Move::Draw, Some(Move::Draw), true);
        assert!(ft > draw, "foundation->tableau is the last resort");
    }

    #[test]
    fn digging_direction_tiebreak() {
        let mut g = blank();
        // Two revealing moves from columns of different heights.
        g.tableau[0] = TableauColumn::new(vec![
            down(Rank::Nine, Suit::Spades),
            up(Rank::Ace, Suit::Hearts),
        ]);
        g.tableau[1] = TableauColumn::new(vec![
            down(Rank::Ten, Suit::Clubs),
            down(Rank::Nine, Suit::Clubs),
            up(Rank::Ace, Suit::Diamonds),
        ]);
        let short = Move::TableauToFoundation {
            column: 0,
            foundation: 0,
        };
        let tall = Move::TableauToFoundation {
            column: 1,
            foundation: 1,
        };
        // Larger-first: the taller column (1) sorts before the shorter (0).
        assert!(move_priority(&g, tall, None, true) < move_priority(&g, short, None, true));
        // Smaller-first inverts it.
        assert!(move_priority(&g, short, None, false) < move_priority(&g, tall, None, false));
    }

    #[test]
    fn empty_column_symmetry_keeps_one_target() {
        let mut g = blank();
        // A King on the waste; columns 1..7 empty (column 0 holds nothing either).
        g.waste = crate::Waste::new(vec![up(Rank::King, Suit::Spades)]);
        // All 7 columns empty -> a King-to-empty move exists for each.
        let moves = vec![
            Move::WasteToTableau { column: 0 },
            Move::WasteToTableau { column: 3 },
            Move::WasteToTableau { column: 6 },
        ];
        let pruned = apply_empty_column_symmetry(&g, moves);
        assert_eq!(pruned, vec![Move::WasteToTableau { column: 0 }]);
    }
}
