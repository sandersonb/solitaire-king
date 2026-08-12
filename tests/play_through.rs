//! Integration test: deal a fixed seed and drive the public API with a
//! deterministic greedy player, asserting core invariants and reproducibility.

use klondike::{legal_moves, GameConfig, GameState, Move};

/// Total cards visible across every pile — must always be 52, all unique.
fn card_census(g: &GameState) -> usize {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut count = 0;
    let mut note = |cards: &[klondike::Card]| {
        for c in cards {
            assert!(seen.insert((c.rank, c.suit)), "duplicate card {c:?}");
            count += 1;
        }
    };
    note(g.stock.cards());
    note(g.waste.cards());
    for f in &g.foundations {
        note(f.cards());
    }
    for col in &g.tableau {
        note(col.cards());
    }
    count
}

/// Pick a move by a deterministic priority: foundations first (progress),
/// then waste/tableau shuffles, then draw, then recycle.
fn choose(moves: &[Move]) -> Option<Move> {
    fn rank(m: &Move) -> u8 {
        match m {
            Move::TableauToFoundation { .. } => 0,
            Move::WasteToFoundation { .. } => 1,
            Move::WasteToTableau { .. } => 2,
            Move::TableauToTableau { .. } => 3,
            Move::FoundationToTableau { .. } => 5, // avoid undoing progress
            Move::Draw => 4,
            Move::Recycle => 6,
        }
    }
    moves.iter().min_by_key(|m| rank(m)).copied()
}

/// Run the greedy player to a natural stopping point (or a step cap) and return
/// the final state.
fn play(seed: u64, config: GameConfig) -> GameState {
    let mut g = GameState::new_with_seed(seed, config);
    assert_eq!(card_census(&g), 52);

    for _ in 0..2000 {
        if g.is_won() {
            break;
        }
        let moves = legal_moves(&g);
        let Some(mv) = choose(&moves) else { break };
        // Stop looping once the only progress left is repeatedly recycling.
        if matches!(mv, Move::Recycle) && g.recycles_done() >= 3 {
            break;
        }
        g.apply(mv).expect("a legal move must apply cleanly");
        // Invariants after every move.
        assert_eq!(card_census(&g), 52, "cards must be conserved");
        assert!(g.current_score() >= 0, "score is never negative");
        for col in &g.tableau {
            assert!(col.invariant_holds(), "tableau invariant must hold");
        }
    }
    g
}

#[test]
fn greedy_playthrough_preserves_invariants() {
    let g = play(20240101, GameConfig::default());
    // Sanity: we made some progress or at least ended in a consistent state.
    assert_eq!(card_census(&g), 52);
    assert!(g.current_score() >= 0);
}

#[test]
fn playthrough_is_reproducible() {
    let cfg = GameConfig::default();
    let a = play(777, cfg);
    let b = play(777, cfg);
    assert_eq!(
        a, b,
        "same seed + same strategy must yield an identical game"
    );
    assert_eq!(a.current_score(), b.current_score());
}

#[test]
fn scripted_opening_scores_expected_points() {
    // Deal a fixed seed and play the first legal foundation/tableau move we can,
    // asserting the score reflects the Windows Standard values.
    let mut g = GameState::new_with_seed(12345, GameConfig::default());
    let start = g.current_score();
    assert_eq!(start, 0);

    // Draw until we can make at least one scoring move or the stock cycles.
    let mut scored = false;
    for _ in 0..100 {
        let moves = legal_moves(&g);
        // Look for any move that yields points (to foundation, or a flip-inducing move).
        if let Some(mv) = moves.iter().find(|m| {
            matches!(
                m,
                Move::WasteToFoundation { .. } | Move::TableauToFoundation { .. }
            )
        }) {
            g.apply(*mv).unwrap();
            scored = true;
            break;
        }
        // Otherwise progress the game deterministically.
        let Some(mv) = choose(&moves) else { break };
        if matches!(mv, Move::Recycle) && g.recycles_done() >= 2 {
            break;
        }
        g.apply(mv).unwrap();
    }

    if scored {
        // A foundation move is worth +10 (possibly plus a +5 flip); never zero.
        assert!(
            g.current_score() >= 10,
            "a foundation move should score at least 10"
        );
    }
    assert_eq!(card_census(&g), 52);
}
