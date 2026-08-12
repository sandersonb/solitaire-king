//! Automatic solver.
//!
//! A depth-first search with heuristics ([`heuristics`]) and a global
//! transposition table ([`table`]) that records proven-winless positions so each
//! distinct position is expanded once. An independent, never-evicted on-path set
//! guarantees termination. A completed search that finds no win reports the deal
//! **proven unwinnable**; a search stopped by the budget reports **inconclusive**.
//! Disabling the table and heuristics ([`SolveOptions::baseline`]) reproduces the
//! naive per-path baseline. Positions are keyed by the reusable [`encode`] key.

pub mod classify;
pub mod encode;
mod heuristics;
mod search;
mod table;
mod zobrist;

use std::time::Duration;

use crate::{GameConfig, Move};

pub use encode::{encode, PositionKey};
pub use search::{solve, solve_state};
pub use zobrist::zobrist;

/// Limits that bound the (exponential) search. The search stops at the first win
/// or when a limit is reached.
#[derive(Debug, Clone, Copy)]
pub struct SolveBudget {
    /// Maximum number of expanded nodes, or `None` for unlimited.
    pub max_nodes: Option<u64>,
    /// Maximum wall-clock time, or `None` for unlimited.
    pub max_time: Option<Duration>,
}

impl Default for SolveBudget {
    /// Defaults: 10,000,000 nodes and 15 seconds.
    fn default() -> Self {
        SolveBudget {
            max_nodes: Some(10_000_000),
            max_time: Some(Duration::from_secs(15)),
        }
    }
}

/// Which pruning rules and heuristics are active. `Default` is the *useful*
/// (heuristic-on) configuration; disabling the heuristics reproduces the naive
/// brute-force baseline.
#[derive(Debug, Clone, Copy)]
pub struct SolveOptions {
    /// Skip no-op moves (provably safe). On by default.
    pub no_op_pruning: bool,
    /// Collapse equivalent waste destinations (experimental). Off by default.
    pub equivalence_pruning: bool,
    /// Force provably-safe foundation auto-moves. On by default.
    pub safe_automoves: bool,
    /// Order candidate moves by the priority heuristic. On by default.
    pub move_ordering: bool,
    /// Only try one empty column when placing a King (safe). On by default.
    pub empty_column_symmetry: bool,
    /// Digging tie-break: prefer revealing in the larger source column. On by default.
    pub dig_larger_first: bool,
    /// Use the global transposition table (closed set). On by default.
    pub transposition_table: bool,
    /// Maximum number of entries the transposition table retains before evicting.
    pub max_table_entries: usize,
    /// How positions are keyed (Zobrist hash by default; exact bytes for validation).
    pub key: KeyStrategy,
}

/// How the search keys positions in its table and on-path set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStrategy {
    /// 128-bit Zobrist hash — compact and fast (default).
    Zobrist,
    /// Exact byte encoding — collision-free reference used for validation.
    ExactBytes,
}

/// Default transposition-table capacity (entries).
pub const DEFAULT_MAX_TABLE_ENTRIES: usize = 4_000_000;

impl Default for SolveOptions {
    fn default() -> Self {
        SolveOptions {
            no_op_pruning: true,
            equivalence_pruning: false,
            safe_automoves: true,
            move_ordering: true,
            empty_column_symmetry: true,
            dig_larger_first: true,
            transposition_table: true,
            max_table_entries: DEFAULT_MAX_TABLE_ENTRIES,
            key: KeyStrategy::Zobrist,
        }
    }
}

impl SolveOptions {
    /// The naive per-path baseline: heuristics and the transposition table off,
    /// leaving only no-op + on-path cycle pruning.
    pub fn baseline() -> Self {
        SolveOptions {
            no_op_pruning: true,
            equivalence_pruning: false,
            safe_automoves: false,
            move_ordering: false,
            empty_column_symmetry: false,
            dig_larger_first: true,
            transposition_table: false,
            max_table_entries: DEFAULT_MAX_TABLE_ENTRIES,
            key: KeyStrategy::Zobrist,
        }
    }
}

/// The three possible outcomes of a search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// A winning line was found.
    Solvable,
    /// The reachable state space was fully explored within budget with no win.
    Unwinnable,
    /// A budget/limit was hit before the space was exhausted and no win was found.
    Inconclusive,
}

/// The outcome of a search.
#[derive(Debug, Clone)]
pub struct SolveResult {
    /// Whether a win was found within the budget.
    pub solvable: bool,
    /// The winning move-sequence, when solvable (need not be minimal).
    pub moveset: Option<Vec<Move>>,
    /// Number of nodes expanded.
    pub nodes_expanded: u64,
    /// Deepest path reached.
    pub max_depth: usize,
    /// Wall-clock time spent.
    pub elapsed: Duration,
    /// Peak logical memory held by the solver's own structures, in bytes.
    pub peak_logical_bytes: usize,
    /// Peak number of positions held on the search path.
    pub peak_positions: usize,
    /// Number of forced safe foundation auto-moves played.
    pub forced_automoves: u64,
    /// The three-way outcome.
    pub verdict: Verdict,
    /// Peak number of positions retained in the transposition table.
    pub table_entries: usize,
    /// Positions skipped because they were found in the transposition table.
    pub table_hits: u64,
    /// Transposition-table evictions (overwrites of an occupied bucket).
    pub table_evictions: u64,
    /// Whether a budget limit was hit (with no win, the result is inconclusive).
    pub budget_exhausted: bool,
}

impl SolveResult {
    /// Whether the outcome is inconclusive: no win found and a budget limit
    /// stopped the search before the space was exhausted.
    pub fn is_inconclusive(&self) -> bool {
        self.verdict == Verdict::Inconclusive
    }

    /// Whether the deal was proven unwinnable (full search, no win, no cut-off).
    pub fn is_unwinnable(&self) -> bool {
        self.verdict == Verdict::Unwinnable
    }
}

/// One seed's before/after verdict when toggling an optional pruning rule.
#[derive(Debug, Clone, Copy)]
pub struct ValidationOutcome {
    pub seed: u64,
    /// Solvable verdict with the rule enabled.
    pub with_rule: bool,
    /// Solvable verdict with the rule disabled.
    pub without_rule: bool,
}

impl ValidationOutcome {
    /// Whether the rule left the solvable verdict unchanged for this seed.
    pub fn agrees(&self) -> bool {
        self.with_rule == self.without_rule
    }
}

/// Differentially validate the **no-op** rule: for each seed, compare the
/// solvable verdict with no-op pruning on vs. off (same budget). A discrepancy
/// means the rule changed win-findability — evidence it is unsound.
pub fn validate_no_op(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    validate(seeds, config, budget, |opts, on| opts.no_op_pruning = on)
}

/// Differentially validate the **equivalence** rule (no-op stays on in both runs).
pub fn validate_equivalence(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    validate(seeds, config, budget, |opts, on| {
        opts.equivalence_pruning = on
    })
}

/// Differentially validate the **safe auto-move** heuristic against the full
/// heuristic configuration (it must not change the solvable verdict).
pub fn validate_safe_automoves(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    validate(seeds, config, budget, |opts, on| opts.safe_automoves = on)
}

/// Differentially validate **move ordering** (must only reorder, never change the verdict).
pub fn validate_move_ordering(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    validate(seeds, config, budget, |opts, on| opts.move_ordering = on)
}

/// Differentially validate **empty-column symmetry** pruning (must not change the verdict).
pub fn validate_empty_column_symmetry(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    validate(seeds, config, budget, |opts, on| {
        opts.empty_column_symmetry = on
    })
}

/// Differentially validate the **transposition table** (must not change the verdict).
pub fn validate_transposition_table(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    validate(seeds, config, budget, |opts, on| {
        opts.transposition_table = on
    })
}

/// Differentially validate the **key strategy**: Zobrist vs exact-byte keys must
/// reach the same solvable verdict (a hash collision would show up as a mismatch).
pub fn validate_key_strategy(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
) -> Vec<ValidationOutcome> {
    seeds
        .iter()
        .map(|&seed| {
            let zobrist = SolveOptions {
                key: KeyStrategy::Zobrist,
                ..SolveOptions::default()
            };
            let exact = SolveOptions {
                key: KeyStrategy::ExactBytes,
                ..SolveOptions::default()
            };
            ValidationOutcome {
                seed,
                with_rule: solve(seed, config, budget, zobrist).solvable,
                without_rule: solve(seed, config, budget, exact).solvable,
            }
        })
        .collect()
}

fn validate(
    seeds: &[u64],
    config: GameConfig,
    budget: SolveBudget,
    set_rule: impl Fn(&mut SolveOptions, bool),
) -> Vec<ValidationOutcome> {
    seeds
        .iter()
        .map(|&seed| {
            let mut with_opts = SolveOptions::default();
            set_rule(&mut with_opts, true);
            let mut without_opts = SolveOptions::default();
            set_rule(&mut without_opts, false);
            ValidationOutcome {
                seed,
                with_rule: solve(seed, config, budget, with_opts).solvable,
                without_rule: solve(seed, config, budget, without_opts).solvable,
            }
        })
        .collect()
}
