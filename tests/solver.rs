//! Integration tests for the solver, driving only the public API.

use klondike::{
    solve, solve_state, validate_key_strategy, Card, DrawMode, Foundation, GameConfig, GameState,
    KeyStrategy, Rank, SolveBudget, SolveOptions, Stock, Suit, TableauColumn, Verdict,
};

fn up(rank: Rank, suit: Suit) -> Card {
    Card::new(rank, suit).face_up()
}

/// A bare state with empty piles, ready to hand-arrange.
fn blank(config: GameConfig) -> GameState {
    let mut g = GameState::new_with_seed(0, config);
    g.stock = Default::default();
    g.waste = Default::default();
    g.foundations = Default::default();
    g.tableau = Default::default();
    g
}

/// Complete clubs, diamonds, hearts on the foundations (39 cards).
fn complete_three_suits(g: &mut GameState) {
    for (i, suit) in [Suit::Clubs, Suit::Diamonds, Suit::Hearts]
        .into_iter()
        .enumerate()
    {
        let mut f = Foundation::new();
        for r in Rank::ALL {
            f.push(up(r, suit));
        }
        g.foundations[i] = f;
    }
}

fn spades_up_to(g: &mut GameState, count: usize) {
    let mut spades = Foundation::new();
    for r in Rank::ALL.into_iter().take(count) {
        spades.push(up(r, Suit::Spades));
    }
    g.foundations[3] = spades;
}

/// A position a few moves from winning: three suits complete, spades to 10, and
/// K♠/Q♠/J♠ stacked face-up (J♠ on top).
fn near_won() -> GameState {
    let mut g = blank(GameConfig::default());
    complete_three_suits(&mut g);
    spades_up_to(&mut g, 10);
    g.tableau[0] = TableauColumn::new(vec![
        up(Rank::King, Suit::Spades),
        up(Rank::Queen, Suit::Spades),
        up(Rank::Jack, Suit::Spades),
    ]);
    g
}

#[test]
fn solves_near_won_and_moveset_replays_to_a_win() {
    let root = near_won();
    let result = solve_state(&root, SolveBudget::default(), SolveOptions::default());
    assert!(result.solvable, "near-won position should solve");
    let moveset = result.moveset.expect("a winning moveset");

    let mut replay = root.clone();
    for mv in &moveset {
        replay
            .apply(*mv)
            .expect("each move in the winning line must be legal");
    }
    assert!(
        replay.is_won(),
        "replaying the moveset must reach a won state"
    );
}

#[test]
fn tiny_budget_reports_inconclusive_not_a_false_win() {
    let budget = SolveBudget {
        max_nodes: Some(50),
        max_time: None,
    };
    let result = solve(1, GameConfig::default(), budget, SolveOptions::default());
    assert!(!result.solvable);
    assert!(result.budget_exhausted);
    assert!(result.is_inconclusive());
    assert!(result.moveset.is_none());
}

#[test]
fn cycle_detection_terminates_on_a_purely_cyclic_position() {
    // One King on an otherwise-empty board. Run the *pure* baseline with no-op AND
    // symmetry off, so the only moves shuffle the King between empty columns;
    // per-path cycle detection must still make the search terminate.
    let mut g = blank(GameConfig::default());
    g.tableau[0] = TableauColumn::new(vec![up(Rank::King, Suit::Spades)]);

    let opts = SolveOptions {
        no_op_pruning: false,
        ..SolveOptions::baseline()
    };
    let result = solve_state(&g, SolveBudget::default(), opts);
    assert!(!result.solvable);
    assert!(
        !result.budget_exhausted,
        "cycle detection should exhaust the (tiny) space, not the budget"
    );
    assert!(result.nodes_expanded < 100_000);
}

#[test]
fn heuristics_preserve_the_verdict_on_a_completing_position() {
    // near_won is small enough to search fully. Every heuristic configuration
    // must agree that it is solvable (soundness: heuristics never lose a win).
    let root = near_won();
    let budget = SolveBudget::default();
    let configs = [
        SolveOptions::default(),
        SolveOptions::baseline(),
        SolveOptions {
            safe_automoves: false,
            ..Default::default()
        },
        SolveOptions {
            move_ordering: false,
            ..Default::default()
        },
        SolveOptions {
            empty_column_symmetry: false,
            ..Default::default()
        },
        SolveOptions {
            dig_larger_first: false,
            ..Default::default()
        },
    ];
    for opts in configs {
        assert!(
            solve_state(&root, budget, opts).solvable,
            "every configuration must still solve near_won: {opts:?}"
        );
    }
}

#[test]
fn heuristics_reduce_work_versus_baseline() {
    let root = near_won();
    let budget = SolveBudget::default();
    let heuristic = solve_state(&root, budget, SolveOptions::default());
    let baseline = solve_state(&root, budget, SolveOptions::baseline());
    assert!(heuristic.solvable && baseline.solvable);
    assert!(
        heuristic.nodes_expanded <= baseline.nodes_expanded,
        "heuristics ({}) should not expand more than baseline ({})",
        heuristic.nodes_expanded,
        baseline.nodes_expanded
    );
    assert!(
        heuristic.forced_automoves > 0,
        "safe auto-moves should fire on near_won"
    );
}

#[test]
fn no_op_pruning_never_expands_more_on_the_baseline() {
    let root = near_won();
    let budget = SolveBudget::default();
    let with = solve_state(&root, budget, SolveOptions::baseline()); // no-op on
    let without = solve_state(
        &root,
        budget,
        SolveOptions {
            no_op_pruning: false,
            ..SolveOptions::baseline()
        },
    );
    assert_eq!(with.solvable, without.solvable);
    assert!(with.nodes_expanded <= without.nodes_expanded);
}

/// A deadlocked position: seven columns of non-stackable, non-Ace cards, empty
/// stock/waste and no empty columns — zero legal moves, so it is unwinnable.
fn deadlocked() -> GameState {
    let mut g = blank(GameConfig::default());
    let cards = [
        (Rank::Five, Suit::Spades),
        (Rank::Five, Suit::Hearts),
        (Rank::Five, Suit::Diamonds),
        (Rank::Five, Suit::Clubs),
        (Rank::Seven, Suit::Spades),
        (Rank::Seven, Suit::Hearts),
        (Rank::Seven, Suit::Diamonds),
    ];
    for (i, (r, s)) in cards.into_iter().enumerate() {
        g.tableau[i] = TableauColumn::new(vec![up(r, s)]);
    }
    g
}

#[test]
fn proves_a_deadlock_unwinnable() {
    let result = solve_state(
        &deadlocked(),
        SolveBudget::default(),
        SolveOptions::default(),
    );
    assert_eq!(result.verdict, Verdict::Unwinnable);
    assert!(result.is_unwinnable());
    assert!(
        !result.budget_exhausted,
        "a full search, not a budget cut-off"
    );
    assert!(result.moveset.is_none());
}

#[test]
fn full_deal_tiny_budget_is_inconclusive_not_unwinnable() {
    let budget = SolveBudget {
        max_nodes: Some(50),
        max_time: None,
    };
    let result = solve(1, GameConfig::default(), budget, SolveOptions::default());
    assert_eq!(result.verdict, Verdict::Inconclusive);
    assert!(!result.is_unwinnable(), "a budget cut-off is never a proof");
}

#[test]
fn table_on_and_off_agree_and_table_does_not_expand_more() {
    for root in [near_won(), deadlocked()] {
        let budget = SolveBudget::default();
        let on = solve_state(&root, budget, SolveOptions::default());
        let off = solve_state(
            &root,
            budget,
            SolveOptions {
                transposition_table: false,
                ..SolveOptions::default()
            },
        );
        assert_eq!(
            on.verdict, off.verdict,
            "the table must not change the verdict"
        );
        assert!(on.nodes_expanded <= off.nodes_expanded);
    }
}

#[test]
fn tiny_table_capacity_is_still_sound() {
    // A capacity-1 table evicts constantly; the verdict must be unchanged.
    let root = near_won();
    let budget = SolveBudget::default();
    let big = solve_state(&root, budget, SolveOptions::default());
    let tiny = solve_state(
        &root,
        budget,
        SolveOptions {
            max_table_entries: 1,
            ..SolveOptions::default()
        },
    );
    assert_eq!(big.verdict, tiny.verdict);
    assert!(big.solvable && tiny.solvable);
}

#[test]
fn zobrist_and_exact_keys_agree_on_completing_positions() {
    let budget = SolveBudget::default();
    for root in [near_won(), deadlocked()] {
        let zobrist = solve_state(
            &root,
            budget,
            SolveOptions {
                key: KeyStrategy::Zobrist,
                ..SolveOptions::default()
            },
        );
        let exact = solve_state(
            &root,
            budget,
            SolveOptions {
                key: KeyStrategy::ExactBytes,
                ..SolveOptions::default()
            },
        );
        assert_eq!(
            zobrist.verdict, exact.verdict,
            "Zobrist and exact-byte keys must agree"
        );
    }
}

#[test]
fn key_strategy_differential_agrees_across_seeds() {
    // Easy seeds that solve quickly: Zobrist and exact keys must agree.
    let seeds = [2u64, 3, 4];
    let budget = SolveBudget {
        max_nodes: Some(200_000),
        max_time: Some(std::time::Duration::from_secs(4)),
    };
    for o in validate_key_strategy(&seeds, GameConfig::default(), budget) {
        assert!(o.agrees(), "key strategies disagreed for seed {}", o.seed);
    }
}

#[test]
fn heuristics_solve_a_deal_with_a_stock_via_forced_automoves() {
    // Three suits complete, spades to 7; 8♠ face-up on a column and 9..K♠ in the
    // stock (draw-one). The heuristic solver wins by forcing safe auto-moves and
    // drawing — end-to-end proof that safe automoves + draws cooperate.
    let config = GameConfig {
        draw_mode: DrawMode::One,
        ..GameConfig::default()
    };
    let mut g = blank(config);
    complete_three_suits(&mut g);
    spades_up_to(&mut g, 7);
    g.tableau[0] = TableauColumn::new(vec![up(Rank::Eight, Suit::Spades)]);
    // Stock top is the last element → draw order 9,10,J,Q,K.
    g.stock = Stock::new(vec![
        up(Rank::King, Suit::Spades),
        up(Rank::Queen, Suit::Spades),
        up(Rank::Jack, Suit::Spades),
        up(Rank::Ten, Suit::Spades),
        up(Rank::Nine, Suit::Spades),
    ]);

    let budget = SolveBudget {
        max_nodes: Some(5_000),
        max_time: None,
    };
    let result = solve_state(&g, budget, SolveOptions::default());
    assert!(result.solvable, "heuristic solver should win this deal");
    assert!(
        result.forced_automoves >= 6,
        "8♠..K♠ are all safe auto-moves"
    );

    let mut replay = g.clone();
    for mv in result.moveset.unwrap() {
        replay.apply(mv).expect("legal");
    }
    assert!(replay.is_won());
}
