//! The `--solve` CLI mode: map flags to a budget/options, run the solver, and
//! print a readable report. The mapping and formatting are pure and tested.

use std::time::Duration;

use klondike::{
    solve, DrawMode, GameConfig, KeyStrategy, SolveBudget, SolveOptions, SolveResult, Verdict,
    DEFAULT_MAX_TABLE_ENTRIES,
};

/// Build a [`SolveBudget`] from flag values. A value of `0` means "unlimited".
pub fn build_budget(max_nodes: u64, max_time_secs: u64) -> SolveBudget {
    SolveBudget {
        max_nodes: (max_nodes != 0).then_some(max_nodes),
        max_time: (max_time_secs != 0).then(|| Duration::from_secs(max_time_secs)),
    }
}

/// Solver heuristic flags gathered from the CLI.
#[derive(Debug, Clone, Copy, Default)]
pub struct HeuristicFlags {
    pub equivalence: bool,
    pub no_noop: bool,
    /// Disable all heuristics at once (reproduce the naive baseline).
    pub baseline: bool,
    pub no_safe_automoves: bool,
    pub no_ordering: bool,
    pub no_symmetry: bool,
    /// Prefer digging the smaller source column instead of the larger.
    pub dig_smaller: bool,
    /// Disable the global transposition table.
    pub no_transposition: bool,
    /// Transposition-table capacity in entries (0 = use the default).
    pub max_table_entries: usize,
    /// Use exact byte keys instead of the default Zobrist hash.
    pub exact_keys: bool,
}

/// Build [`SolveOptions`] from the heuristic flags. Heuristics and the table are
/// on by default; `--baseline` forces them all off (leaving only no-op + cycle
/// pruning).
pub fn build_options(f: HeuristicFlags) -> SolveOptions {
    let max_table_entries = if f.max_table_entries == 0 {
        DEFAULT_MAX_TABLE_ENTRIES
    } else {
        f.max_table_entries
    };
    let key = if f.exact_keys {
        KeyStrategy::ExactBytes
    } else {
        KeyStrategy::Zobrist
    };
    if f.baseline {
        return SolveOptions {
            max_table_entries,
            key,
            ..SolveOptions::baseline()
        };
    }
    SolveOptions {
        no_op_pruning: !f.no_noop,
        equivalence_pruning: f.equivalence,
        safe_automoves: !f.no_safe_automoves,
        move_ordering: !f.no_ordering,
        empty_column_symmetry: !f.no_symmetry,
        dig_larger_first: !f.dig_smaller,
        transposition_table: !f.no_transposition,
        max_table_entries,
        key,
    }
}

/// Run the solver and print the report to stdout.
pub fn run(seed: u64, config: GameConfig, budget: SolveBudget, options: SolveOptions) {
    let result = solve(seed, config, budget, options);
    print!("{}", format_report(seed, config, &result));
}

/// Format the solver result as a readable multi-line report.
pub fn format_report(seed: u64, config: GameConfig, result: &SolveResult) -> String {
    let draw = match config.draw_mode {
        DrawMode::One => "1",
        DrawMode::Three => "3",
    };
    let redeal = match config.redeal_limit {
        Some(n) => n.to_string(),
        None => "unlimited".to_string(),
    };

    let verdict = match result.verdict {
        Verdict::Solvable => "SOLVABLE",
        Verdict::Unwinnable => "UNWINNABLE (proven — full search, no win)",
        Verdict::Inconclusive => "INCONCLUSIVE (budget hit; not proven unwinnable)",
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Klondike solver — seed {}  (draw {draw}, redeal {redeal})\n",
        klondike::seed::encode(seed)
    ));
    out.push_str(&format!("result: {verdict}\n"));
    out.push_str(&format!("nodes expanded: {}\n", result.nodes_expanded));
    out.push_str(&format!("max depth: {}\n", result.max_depth));
    out.push_str(&format!("elapsed: {:.2?}\n", result.elapsed));
    out.push_str(&format!(
        "peak memory: {} B ({} positions)\n",
        result.peak_logical_bytes, result.peak_positions
    ));
    out.push_str(&format!("forced auto-moves: {}\n", result.forced_automoves));
    out.push_str(&format!(
        "transposition table: {} entries, {} hits, {} evictions\n",
        result.table_entries, result.table_hits, result.table_evictions
    ));

    if let Some(moveset) = &result.moveset {
        out.push_str(&format!("winning moves: {}\n", moveset.len()));
        for (i, mv) in moveset.iter().enumerate() {
            out.push_str(&format!("  {:>3}. {:?}\n", i + 1, mv));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_zero_means_unlimited() {
        let b = build_budget(0, 0);
        assert!(b.max_nodes.is_none());
        assert!(b.max_time.is_none());
        let b = build_budget(500, 7);
        assert_eq!(b.max_nodes, Some(500));
        assert_eq!(b.max_time, Some(Duration::from_secs(7)));
    }

    #[test]
    fn options_mapping() {
        // Defaults: heuristics on, no-op on, equivalence off.
        let o = build_options(HeuristicFlags::default());
        assert!(o.no_op_pruning);
        assert!(!o.equivalence_pruning);
        assert!(o.safe_automoves && o.move_ordering && o.empty_column_symmetry);
        assert!(o.dig_larger_first);

        // Individual disables.
        let o = build_options(HeuristicFlags {
            equivalence: true,
            no_noop: true,
            no_ordering: true,
            dig_smaller: true,
            ..Default::default()
        });
        assert!(!o.no_op_pruning);
        assert!(o.equivalence_pruning);
        assert!(!o.move_ordering);
        assert!(!o.dig_larger_first);
        assert!(o.safe_automoves && o.empty_column_symmetry);
    }

    #[test]
    fn exact_keys_flag_maps_to_key_strategy() {
        assert_eq!(
            build_options(HeuristicFlags::default()).key,
            KeyStrategy::Zobrist
        );
        assert_eq!(
            build_options(HeuristicFlags {
                exact_keys: true,
                ..Default::default()
            })
            .key,
            KeyStrategy::ExactBytes
        );
    }

    #[test]
    fn baseline_flag_disables_all_heuristics() {
        let o = build_options(HeuristicFlags {
            baseline: true,
            ..Default::default()
        });
        assert!(o.no_op_pruning); // baseline keeps no-op + cycle pruning
        assert!(!o.safe_automoves);
        assert!(!o.move_ordering);
        assert!(!o.empty_column_symmetry);
        assert!(!o.equivalence_pruning);
    }

    fn sample_result(verdict: Verdict) -> SolveResult {
        let solvable = verdict == Verdict::Solvable;
        SolveResult {
            solvable,
            moveset: solvable.then(|| vec![klondike::Move::Draw]),
            nodes_expanded: 3,
            max_depth: 1,
            elapsed: Duration::from_millis(1200),
            peak_logical_bytes: 210,
            peak_positions: 3,
            forced_automoves: 4,
            verdict,
            table_entries: 100,
            table_hits: 7,
            table_evictions: 2,
            budget_exhausted: verdict == Verdict::Inconclusive,
        }
    }

    #[test]
    fn report_solved_lists_moves_and_stats() {
        let text = format_report(42, GameConfig::default(), &sample_result(Verdict::Solvable));
        assert!(text.contains(&format!("seed {}", klondike::seed::encode(42))));
        assert!(text.contains("result: SOLVABLE"));
        assert!(text.contains("winning moves: 1"));
        assert!(text.contains("forced auto-moves: 4"));
        assert!(text.contains("transposition table: 100 entries, 7 hits, 2 evictions"));
        assert!(text.contains("Draw"));
    }

    #[test]
    fn report_unwinnable_is_distinct_from_inconclusive() {
        let unwinnable = format_report(
            1,
            GameConfig::default(),
            &sample_result(Verdict::Unwinnable),
        );
        assert!(unwinnable.contains("UNWINNABLE (proven"));
        let inconclusive = format_report(
            1,
            GameConfig::default(),
            &sample_result(Verdict::Inconclusive),
        );
        assert!(inconclusive.contains("INCONCLUSIVE"));
        assert!(!inconclusive.contains("winning moves"));
    }
}
