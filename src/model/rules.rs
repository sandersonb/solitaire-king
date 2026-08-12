//! The classic-rules engine: placement rules, legal-move enumeration, and move
//! application (including auto-flip, stock draw/recycle, and scoring).

use crate::model::card::{Card, Rank};
use crate::model::moves::{Move, MoveError};
use crate::model::score::{Score, ScoreEvent};
use crate::model::state::{DrawMode, GameState};

const NUM_FOUNDATIONS: usize = 4;
const NUM_COLUMNS: usize = 7;

/// Whether `moving` may be stacked onto a tableau column whose top card is
/// `dest_top` (`None` for an empty column). An empty column accepts only a King;
/// otherwise the incoming card must be one rank lower and the opposite color.
pub fn can_stack_on_tableau(moving: Card, dest_top: Option<Card>) -> bool {
    match dest_top {
        None => moving.rank == Rank::King,
        Some(top) => {
            top.face_up
                && moving.color() != top.color()
                && top.rank.value() == moving.rank.value() + 1
        }
    }
}

/// Whether `moving` may be placed onto a foundation whose top card is
/// `found_top` (`None` for an empty foundation). An empty foundation accepts
/// only an Ace; otherwise the incoming card must be the same suit and one rank
/// higher.
pub fn can_stack_on_foundation(moving: Card, found_top: Option<Card>) -> bool {
    match found_top {
        None => moving.rank == Rank::Ace,
        Some(top) => moving.suit == top.suit && moving.rank.value() == top.rank.value() + 1,
    }
}

/// Whether `run` (ordered bottom-to-top) is a valid movable tableau sequence:
/// every card face-up, each successive card one rank lower and the opposite
/// color of the one beneath it.
pub fn is_valid_run(run: &[Card]) -> bool {
    run.iter().all(|c| c.face_up)
        && run.windows(2).all(|w| {
            let (lower, upper) = (w[0], w[1]);
            lower.color() != upper.color() && lower.rank.value() == upper.rank.value() + 1
        })
}

/// Enumerate every legal move for `state` under the classic ruleset.
pub fn legal_moves(state: &GameState) -> Vec<Move> {
    let mut moves = Vec::new();

    // Stock: draw when non-empty, else recycle when permitted and the waste
    // has cards to return.
    if !state.stock.is_empty() {
        moves.push(Move::Draw);
    } else if !state.waste.is_empty() && state.recycle_allowed() {
        moves.push(Move::Recycle);
    }

    // Waste top placements.
    if let Some(card) = state.waste.top() {
        for foundation in 0..NUM_FOUNDATIONS {
            if can_stack_on_foundation(card, state.foundations[foundation].top()) {
                moves.push(Move::WasteToFoundation { foundation });
            }
        }
        for column in 0..NUM_COLUMNS {
            if can_stack_on_tableau(card, state.tableau[column].top()) {
                moves.push(Move::WasteToTableau { column });
            }
        }
    }

    // Tableau top → foundation.
    for column in 0..NUM_COLUMNS {
        if let Some(card) = state.tableau[column].top() {
            for foundation in 0..NUM_FOUNDATIONS {
                if can_stack_on_foundation(card, state.foundations[foundation].top()) {
                    moves.push(Move::TableauToFoundation { column, foundation });
                }
            }
        }
    }

    // Tableau run → tableau. For each source, try every valid face-up run length
    // and every other destination column.
    for from in 0..NUM_COLUMNS {
        let run = state.tableau[from].face_up_run();
        for count in 1..=run.len() {
            // The deepest card of the moved run (its bottom) lands on the destination.
            let bottom = run[run.len() - count];
            if !is_valid_run(&run[run.len() - count..]) {
                continue;
            }
            for to in 0..NUM_COLUMNS {
                if to == from {
                    continue;
                }
                if can_stack_on_tableau(bottom, state.tableau[to].top()) {
                    moves.push(Move::TableauToTableau { from, to, count });
                }
            }
        }
    }

    // Foundation top → tableau.
    for foundation in 0..NUM_FOUNDATIONS {
        if let Some(card) = state.foundations[foundation].top() {
            for column in 0..NUM_COLUMNS {
                if can_stack_on_tableau(card, state.tableau[column].top()) {
                    moves.push(Move::FoundationToTableau { foundation, column });
                }
            }
        }
    }

    moves
}

/// A token capturing what `undo_move` needs to reverse a move. `Copy`, no heap.
///
/// `drawn` is how many cards a `Draw` moved; `flipped` whether an auto-flip
/// fired; `prev_score`/`prev_recycles` the values to restore.
#[derive(Debug, Clone, Copy)]
pub struct Undo {
    drawn: u8,
    flipped: bool,
    prev_score: Score,
    prev_recycles: u32,
}

/// Apply `mv` to `state`, returning an [`Undo`] token that reverses it. Same
/// effects as [`apply_move`]; illegal moves are rejected leaving the state
/// unchanged (no token produced).
pub fn apply_undoable(state: &mut GameState, mv: Move) -> Result<Undo, MoveError> {
    let prev_score = state.score();
    let prev_recycles = state.recycles_done();
    let (drawn, flipped) = match mv {
        Move::Draw => apply_draw(state)?,
        Move::Recycle => apply_recycle(state)?,
        Move::WasteToFoundation { foundation } => apply_waste_to_foundation(state, foundation)?,
        Move::WasteToTableau { column } => apply_waste_to_tableau(state, column)?,
        Move::TableauToTableau { from, to, count } => {
            apply_tableau_to_tableau(state, from, to, count)?
        }
        Move::TableauToFoundation { column, foundation } => {
            apply_tableau_to_foundation(state, column, foundation)?
        }
        Move::FoundationToTableau { foundation, column } => {
            apply_foundation_to_tableau(state, foundation, column)?
        }
    };
    Ok(Undo {
        drawn,
        flipped,
        prev_score,
        prev_recycles,
    })
}

/// Apply `mv` to `state`. On success the state is mutated (and scored). On any
/// error the state is left exactly as it was.
pub fn apply_move(state: &mut GameState, mv: Move) -> Result<(), MoveError> {
    apply_undoable(state, mv).map(|_| ())
}

/// Reverse a move previously applied with [`apply_undoable`], restoring the
/// exact prior state. Allocates nothing.
pub fn undo_move(state: &mut GameState, mv: Move, undo: Undo) {
    match mv {
        Move::Draw => {
            for _ in 0..undo.drawn {
                let mut card = state.waste.take_top().expect("drawn card present");
                card.flip_down();
                state.stock.push(card);
            }
        }
        Move::Recycle => {
            for mut card in state.stock.drain_top_first() {
                card.flip_up();
                state.waste.push(card);
            }
        }
        Move::WasteToFoundation { foundation } => {
            let card = state.foundations[foundation]
                .take_top()
                .expect("foundation card");
            state.waste.push(card);
        }
        Move::WasteToTableau { column } => {
            let card = state.tableau[column].take_top().expect("tableau card");
            state.waste.push(card);
        }
        Move::TableauToTableau { from, to, count } => {
            if undo.flipped {
                state.tableau[from].flip_top_down();
            }
            let run = state.tableau[to]
                .take_run(count)
                .expect("moved run present");
            state.tableau[from].push_run(run);
        }
        Move::TableauToFoundation { column, foundation } => {
            if undo.flipped {
                state.tableau[column].flip_top_down();
            }
            let card = state.foundations[foundation]
                .take_top()
                .expect("foundation card");
            state.tableau[column].push(card);
        }
        Move::FoundationToTableau { foundation, column } => {
            let card = state.tableau[column].take_top().expect("tableau card");
            state.foundations[foundation].push(card);
        }
    }
    state.restore_score_and_recycles(undo.prev_score, undo.prev_recycles);
}

fn apply_draw(state: &mut GameState) -> Result<(u8, bool), MoveError> {
    if state.stock.is_empty() {
        return Err(MoveError::StockEmpty);
    }
    let n = state.config().draw_mode.count().min(state.stock.len());
    for _ in 0..n {
        let mut card = state.stock.draw().expect("stock non-empty");
        card.flip_up();
        state.waste.push(card);
    }
    Ok((n as u8, false))
}

fn apply_recycle(state: &mut GameState) -> Result<(u8, bool), MoveError> {
    if !state.stock.is_empty() {
        return Err(MoveError::StockNotEmpty);
    }
    if state.waste.is_empty() {
        return Err(MoveError::EmptySource);
    }
    if !state.recycle_allowed() {
        return Err(MoveError::RedealLimitReached);
    }
    // Return the waste to the stock in draw order (top of waste becomes top of stock),
    // turning every card face-down.
    for mut card in state.waste.drain_top_first() {
        card.flip_down();
        state.stock.push(card);
    }
    state.record_recycle();
    // Draw-one recycles are penalized (each is after the first pass); draw-three
    // recycles carry no penalty.
    if state.config().draw_mode == DrawMode::One {
        state.score_mut().apply(ScoreEvent::RecycleDrawOne);
    }
    Ok((0, false))
}

fn apply_waste_to_foundation(
    state: &mut GameState,
    foundation: usize,
) -> Result<(u8, bool), MoveError> {
    if foundation >= NUM_FOUNDATIONS {
        return Err(MoveError::IndexOutOfRange);
    }
    let card = state.waste.top().ok_or(MoveError::EmptySource)?;
    if !can_stack_on_foundation(card, state.foundations[foundation].top()) {
        return Err(MoveError::IllegalFoundationPlacement);
    }
    let card = state.waste.take_top().expect("waste non-empty");
    state.foundations[foundation].push(card);
    state.score_mut().apply(ScoreEvent::WasteToFoundation);
    Ok((0, false))
}

fn apply_waste_to_tableau(state: &mut GameState, column: usize) -> Result<(u8, bool), MoveError> {
    if column >= NUM_COLUMNS {
        return Err(MoveError::IndexOutOfRange);
    }
    let card = state.waste.top().ok_or(MoveError::EmptySource)?;
    if !can_stack_on_tableau(card, state.tableau[column].top()) {
        return Err(MoveError::IllegalTableauPlacement);
    }
    let card = state.waste.take_top().expect("waste non-empty");
    state.tableau[column].push(card);
    state.score_mut().apply(ScoreEvent::WasteToTableau);
    Ok((0, false))
}

fn apply_tableau_to_tableau(
    state: &mut GameState,
    from: usize,
    to: usize,
    count: usize,
) -> Result<(u8, bool), MoveError> {
    if from >= NUM_COLUMNS || to >= NUM_COLUMNS {
        return Err(MoveError::IndexOutOfRange);
    }
    if from == to {
        return Err(MoveError::IllegalTableauPlacement);
    }
    let run = state.tableau[from].face_up_run();
    if count == 0 || count > run.len() {
        return Err(MoveError::InvalidRun);
    }
    let moving = &run[run.len() - count..];
    if !is_valid_run(moving) {
        return Err(MoveError::InvalidRun);
    }
    let bottom = moving[0];
    if !can_stack_on_tableau(bottom, state.tableau[to].top()) {
        return Err(MoveError::IllegalTableauPlacement);
    }
    let run = state.tableau[from].take_run(count).expect("validated run");
    state.tableau[to].push_run(run);
    // Auto-flip the newly exposed card in the source column.
    let flipped = state.tableau[from].flip_top_if_face_down();
    if flipped {
        state.score_mut().apply(ScoreEvent::FlipTableauCard);
    }
    Ok((0, flipped))
}

fn apply_tableau_to_foundation(
    state: &mut GameState,
    column: usize,
    foundation: usize,
) -> Result<(u8, bool), MoveError> {
    if column >= NUM_COLUMNS || foundation >= NUM_FOUNDATIONS {
        return Err(MoveError::IndexOutOfRange);
    }
    let card = state.tableau[column].top().ok_or(MoveError::EmptySource)?;
    if !can_stack_on_foundation(card, state.foundations[foundation].top()) {
        return Err(MoveError::IllegalFoundationPlacement);
    }
    let card = state.tableau[column].take_top().expect("column non-empty");
    state.foundations[foundation].push(card);
    state.score_mut().apply(ScoreEvent::TableauToFoundation);
    let flipped = state.tableau[column].flip_top_if_face_down();
    if flipped {
        state.score_mut().apply(ScoreEvent::FlipTableauCard);
    }
    Ok((0, flipped))
}

fn apply_foundation_to_tableau(
    state: &mut GameState,
    foundation: usize,
    column: usize,
) -> Result<(u8, bool), MoveError> {
    if foundation >= NUM_FOUNDATIONS || column >= NUM_COLUMNS {
        return Err(MoveError::IndexOutOfRange);
    }
    let card = state.foundations[foundation]
        .top()
        .ok_or(MoveError::EmptySource)?;
    if !can_stack_on_tableau(card, state.tableau[column].top()) {
        return Err(MoveError::IllegalTableauPlacement);
    }
    let card = state.foundations[foundation]
        .take_top()
        .expect("foundation non-empty");
    state.tableau[column].push(card);
    state.score_mut().apply(ScoreEvent::FoundationToTableau);
    Ok((0, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::card::{Card, Rank, Suit};
    use crate::model::pile::{Foundation, Stock, TableauColumn, Waste};
    use crate::model::state::{DrawMode, GameConfig, GameState};

    fn up(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit).face_up()
    }
    fn down(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    /// Build a bare state we can hand-arrange for focused rule tests.
    fn blank(config: GameConfig) -> GameState {
        let mut g = GameState::new_with_seed(0, config);
        g.stock = Stock::default();
        g.waste = Waste::default();
        g.foundations = Default::default();
        g.tableau = Default::default();
        g
    }

    #[test]
    fn tableau_placement_rules() {
        // Red six on black seven: legal.
        assert!(can_stack_on_tableau(
            up(Rank::Six, Suit::Hearts),
            Some(up(Rank::Seven, Suit::Spades))
        ));
        // Same color: illegal.
        assert!(!can_stack_on_tableau(
            up(Rank::Six, Suit::Diamonds),
            Some(up(Rank::Seven, Suit::Hearts))
        ));
        // Empty column: only a King.
        assert!(can_stack_on_tableau(up(Rank::King, Suit::Clubs), None));
        assert!(!can_stack_on_tableau(up(Rank::Queen, Suit::Clubs), None));
    }

    #[test]
    fn foundation_placement_rules() {
        assert!(can_stack_on_foundation(up(Rank::Ace, Suit::Hearts), None));
        assert!(!can_stack_on_foundation(up(Rank::Two, Suit::Hearts), None));
        assert!(can_stack_on_foundation(
            up(Rank::Five, Suit::Hearts),
            Some(up(Rank::Four, Suit::Hearts))
        ));
        // Wrong suit.
        assert!(!can_stack_on_foundation(
            up(Rank::Five, Suit::Spades),
            Some(up(Rank::Four, Suit::Hearts))
        ));
        // Not the next rank.
        assert!(!can_stack_on_foundation(
            up(Rank::Six, Suit::Hearts),
            Some(up(Rank::Four, Suit::Hearts))
        ));
    }

    #[test]
    fn illegal_move_is_a_no_op() {
        let mut g = blank(GameConfig::default());
        g.tableau[0] = TableauColumn::new(vec![up(Rank::Five, Suit::Hearts)]);
        let before = g.clone();
        // Cannot put a red five on a red... there's nothing to put anyway; try
        // an illegal foundation move.
        let err = apply_move(
            &mut g,
            Move::TableauToFoundation {
                column: 0,
                foundation: 0,
            },
        );
        assert_eq!(err, Err(MoveError::IllegalFoundationPlacement));
        assert_eq!(g, before, "state must be unchanged after an illegal move");
    }

    #[test]
    fn tableau_to_foundation_scores_and_autoflips() {
        let mut g = blank(GameConfig::default());
        // Column: [face-down 9♠, face-up A♥]. Moving the Ace to an empty
        // foundation should score +10 and flip the 9♠ (+5).
        g.tableau[0] = TableauColumn::new(vec![
            down(Rank::Nine, Suit::Spades),
            up(Rank::Ace, Suit::Hearts),
        ]);
        apply_move(
            &mut g,
            Move::TableauToFoundation {
                column: 0,
                foundation: 0,
            },
        )
        .unwrap();
        assert_eq!(g.foundations[0].top().unwrap().rank, Rank::Ace);
        assert!(
            g.tableau[0].top().unwrap().face_up,
            "9♠ should be flipped up"
        );
        assert_eq!(g.score().event_points(), 15); // 10 + 5 flip
    }

    #[test]
    fn foundation_to_tableau_penalizes() {
        let mut g = blank(GameConfig::default());
        let mut f = Foundation::new();
        f.push(up(Rank::Ace, Suit::Spades));
        f.push(up(Rank::Two, Suit::Spades));
        g.foundations[0] = f;
        // Two of Spades (black) onto a red Three.
        g.tableau[0] = TableauColumn::new(vec![up(Rank::Three, Suit::Hearts)]);
        apply_move(
            &mut g,
            Move::FoundationToTableau {
                foundation: 0,
                column: 0,
            },
        )
        .unwrap();
        // Score was 0, -15 clamps to 0.
        assert_eq!(g.score().event_points(), 0);
        assert_eq!(g.tableau[0].top().unwrap().rank, Rank::Two);
    }

    #[test]
    fn recycle_respects_limit_and_penalty() {
        // Draw-one with a redeal limit of 1.
        let cfg = GameConfig {
            draw_mode: DrawMode::One,
            redeal_limit: Some(1),
            timed: false,
        };
        let mut g = blank(cfg);
        g.waste = Waste::new(vec![up(Rank::Ace, Suit::Clubs), up(Rank::Two, Suit::Clubs)]);
        // First recycle allowed, penalized -100 (draw-one) clamped to 0.
        apply_move(&mut g, Move::Recycle).unwrap();
        assert_eq!(g.recycles_done(), 1);
        assert!(g.waste.is_empty());
        assert_eq!(g.stock.len(), 2);
        // Draw the stock empty again, then a second recycle is blocked by the limit.
        apply_move(&mut g, Move::Draw).unwrap();
        apply_move(&mut g, Move::Draw).unwrap();
        assert!(g.stock.is_empty());
        let err = apply_move(&mut g, Move::Recycle);
        assert_eq!(err, Err(MoveError::RedealLimitReached));
    }

    #[test]
    fn draw_three_moves_three_cards() {
        let cfg = GameConfig {
            draw_mode: DrawMode::Three,
            ..Default::default()
        };
        let mut g = blank(cfg);
        g.stock = Stock::new(vec![
            down(Rank::Ace, Suit::Clubs),
            down(Rank::Two, Suit::Clubs),
            down(Rank::Three, Suit::Clubs),
            down(Rank::Four, Suit::Clubs),
        ]);
        apply_move(&mut g, Move::Draw).unwrap();
        assert_eq!(g.waste.len(), 3);
        assert!(g.waste.cards().iter().all(|c| c.face_up));
        assert_eq!(g.stock.len(), 1);
    }

    #[test]
    fn move_a_run_between_columns() {
        let mut g = blank(GameConfig::default());
        // Source column top run: 7♥(red), 6♠(black) — a valid descending run.
        g.tableau[0] = TableauColumn::new(vec![
            down(Rank::King, Suit::Clubs),
            up(Rank::Seven, Suit::Hearts),
            up(Rank::Six, Suit::Spades),
        ]);
        // Destination top: 8♠(black) accepts red 7.
        g.tableau[1] = TableauColumn::new(vec![up(Rank::Eight, Suit::Spades)]);
        apply_move(
            &mut g,
            Move::TableauToTableau {
                from: 0,
                to: 1,
                count: 2,
            },
        )
        .unwrap();
        assert_eq!(g.tableau[1].len(), 3);
        assert_eq!(g.tableau[1].top().unwrap().rank, Rank::Six);
        // Source exposed the King and auto-flipped it (+5).
        assert!(g.tableau[0].top().unwrap().face_up);
        assert_eq!(g.score().event_points(), 5);
    }

    #[test]
    fn win_detection() {
        let mut g = blank(GameConfig::default());
        assert!(!g.is_won());
        for (i, suit) in Suit::ALL.into_iter().enumerate() {
            let mut f = Foundation::new();
            for rank in Rank::ALL {
                f.push(up(rank, suit));
            }
            g.foundations[i] = f;
        }
        assert!(g.is_won());
    }

    #[test]
    fn apply_undo_is_identity_and_matches_clone_over_a_walk() {
        // Over several deals, at each reached state, verify for EVERY legal move
        // that (a) apply_undoable then undo_move restores the exact prior state,
        // and (b) apply_undoable equals cloning and calling apply_move.
        for seed in [1u64, 2, 7, 42, 2024, 12345] {
            let mut state = GameState::new_with_seed(seed, GameConfig::default());
            for step in 0..60 {
                let before = state.clone();
                let moves = legal_moves(&state);
                if moves.is_empty() {
                    break;
                }
                for &mv in &moves {
                    // make == clone-and-apply
                    let mut cloned = before.clone();
                    apply_move(&mut cloned, mv).expect("legal move applies to clone");
                    let undo = apply_undoable(&mut state, mv).expect("legal move applies");
                    assert_eq!(state, cloned, "make must equal clone-and-apply: {mv:?}");
                    // undo restores exactly
                    undo_move(&mut state, mv, undo);
                    assert_eq!(
                        state, before,
                        "undo must restore exact state (seed {seed}, step {step}, {mv:?})"
                    );
                }
                // Advance to a new state; prefer the last move (tableau/foundation
                // moves sort later) to dig into the deal and exercise auto-flips.
                let mv = *moves.last().unwrap();
                apply_undoable(&mut state, mv).expect("advance move applies");
            }
        }
    }

    #[test]
    fn undo_reverses_auto_flip() {
        let mut g = blank(GameConfig::default());
        // Column: face-down 9♠ beneath a face-up A♥. Moving the Ace up flips 9♠.
        g.tableau[0] = TableauColumn::new(vec![
            down(Rank::Nine, Suit::Spades),
            up(Rank::Ace, Suit::Hearts),
        ]);
        let before = g.clone();
        let mv = Move::TableauToFoundation {
            column: 0,
            foundation: 0,
        };
        let undo = apply_undoable(&mut g, mv).unwrap();
        assert!(
            g.tableau[0].top().unwrap().face_up,
            "9♠ flipped up by the move"
        );
        undo_move(&mut g, mv, undo);
        assert_eq!(g, before, "undo restores the face-down 9♠ and the Ace");
    }

    #[test]
    fn undo_reverses_recycle_with_score_and_count() {
        let cfg = GameConfig {
            draw_mode: DrawMode::One,
            ..GameConfig::default()
        };
        let mut g = blank(cfg);
        g.waste = Waste::new(vec![up(Rank::Ace, Suit::Clubs), up(Rank::Two, Suit::Clubs)]);
        let before = g.clone();
        let undo = apply_undoable(&mut g, Move::Recycle).unwrap();
        assert_eq!(g.recycles_done(), 1);
        assert!(g.stock.len() == 2 && g.waste.is_empty());
        undo_move(&mut g, Move::Recycle, undo);
        assert_eq!(
            g, before,
            "undo restores stock/waste, recycle count, and score"
        );
    }
}
