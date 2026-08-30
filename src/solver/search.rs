//! The depth-first search: heuristics, an independent on-path cycle guard, and a
//! global transposition table (closed set). It explores by **make/unmake** — a
//! single mutable `GameState` is mutated down each branch and undone on
//! backtrack, with no per-child clone. Positions are keyed by a 128-bit Zobrist
//! hash by default, or the exact byte encoding for validation.
//!
//! Full caching carries a theoretical graph-history-interaction risk (a cycle
//! prune could make a winnable position look winless); it is mitigated by the
//! differential validators. The on-path set (never evicted) guarantees
//! termination; reported wins are always replay-verifiable.

use std::collections::HashSet;
use std::hash::Hash;
use std::time::Duration;

use crate::model::rules::{apply_undoable, legal_moves, undo_move};
use crate::solver::classify::{apply_equivalence_pruning, column_has_legal_move, no_op_structural};
use crate::solver::encode::{encode, PositionKey};
use crate::solver::heuristics::{apply_empty_column_symmetry, move_priority, safe_move_in};
use crate::solver::table::ClosedTable;
use crate::solver::zobrist::zobrist;
use crate::solver::{KeyStrategy, SolveBudget, SolveOptions, SolveResult, Verdict};
use crate::{GameConfig, GameState, Move};

/// A monotonic clock for the elapsed-time budget. On `wasm32-unknown-unknown`
/// there is no usable monotonic clock (`std::time::Instant::now()` panics), so
/// the wasm implementation is a no-op: elapsed is zero and the time deadline is
/// never reached, leaving the node budget to bound the search.
#[cfg(not(target_arch = "wasm32"))]
mod clock {
    use std::time::{Duration, Instant};

    pub struct Clock(Instant);

    impl Clock {
        pub fn start() -> Self {
            Clock(Instant::now())
        }
        pub fn elapsed(&self) -> Duration {
            self.0.elapsed()
        }
        pub fn reached(&self, max: Duration) -> bool {
            self.0.elapsed() >= max
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod clock {
    use std::time::Duration;

    pub struct Clock;

    impl Clock {
        pub fn start() -> Self {
            Clock
        }
        pub fn elapsed(&self) -> Duration {
            Duration::ZERO
        }
        pub fn reached(&self, _max: Duration) -> bool {
            false
        }
    }
}

use clock::Clock;

/// Solve the deal for `seed`/`config` under the given budget and options.
pub fn solve(
    seed: u64,
    config: GameConfig,
    budget: SolveBudget,
    options: SolveOptions,
) -> SolveResult {
    solve_state(&GameState::new_with_seed(seed, config), budget, options)
}

/// Solve from an explicit starting position — the core entry point. Dispatches
/// on the key strategy, monomorphizing the generic search for each key type.
pub fn solve_state(root: &GameState, budget: SolveBudget, options: SolveOptions) -> SolveResult {
    // The table is built per key type inside each arm, so its `K` matches.
    let cap = options.max_table_entries;
    let with_table = options.transposition_table;
    match options.key {
        KeyStrategy::Zobrist => {
            let closed = with_table.then(|| ClosedTable::with_capacity(cap));
            run(root, budget, options, zobrist, closed).0
        }
        KeyStrategy::ExactBytes => {
            let closed = with_table.then(|| ClosedTable::with_capacity(cap));
            run(root, budget, options, encode, closed).0
        }
    }
}

/// Solve from `root` under a node budget, **reusing the caller's transposition
/// table**. The table is left populated with the proven-winless positions found,
/// so a later call with the same table (from the same or another reachable
/// position) skips positions already proven winless — letting repeated bounded
/// searches make monotonic progress. Keys are exact bytes, so reuse is sound and
/// a proven-unwinnable verdict cannot be a hash collision. There is no time
/// budget (the caller bounds wall-clock across calls); the search is bounded by
/// `node_budget` (`None` = unbounded).
pub fn solve_reusing(
    root: &GameState,
    node_budget: Option<u64>,
    options: SolveOptions,
    table: &mut ClosedTable<PositionKey>,
) -> SolveResult {
    let budget = SolveBudget {
        max_nodes: node_budget,
        max_time: None,
    };
    let mut opts = options;
    opts.key = KeyStrategy::ExactBytes;
    opts.transposition_table = true;
    // Borrow the caller's table by swapping it out and back so the search can own
    // it for the duration of the run.
    let taken = std::mem::replace(table, ClosedTable::with_capacity(1));
    let (result, returned) = run(root, budget, opts, encode, Some(taken));
    *table = returned.expect("reusing search keeps its table");
    result
}

/// Run the search with a concrete key function `key_of`, taking the closed table
/// by value and returning it so it can be reused across calls.
fn run<K, F>(
    root: &GameState,
    budget: SolveBudget,
    options: SolveOptions,
    key_of: F,
    closed: Option<ClosedTable<K>>,
) -> (SolveResult, Option<ClosedTable<K>>)
where
    K: Hash + Eq + Clone,
    F: Fn(&GameState) -> K,
{
    let clock = Clock::start();

    let mut search = Search {
        budget,
        options,
        clock,
        max_time: budget.max_time,
        nodes: 0,
        max_depth: 0,
        path: HashSet::new(),
        peak_positions: 0,
        closed,
        table_hits: 0,
        forced_automoves: 0,
        found: None,
        budget_exhausted: false,
        key_of,
    };

    // One clone: the mutable working state make/unmake threads down the tree.
    let mut state = root.clone();
    let mut moves = Vec::new();
    search.dfs(&mut state, &mut moves);

    let moveset = search.found.take();
    let verdict = if moveset.is_some() {
        Verdict::Solvable
    } else if search.budget_exhausted {
        Verdict::Inconclusive
    } else {
        Verdict::Unwinnable
    };

    let table_entries = search.closed.as_ref().map_or(0, |t| t.peak_entries());
    let table_evictions = search.closed.as_ref().map_or(0, |t| t.evictions());
    let key_bytes = std::mem::size_of::<K>();
    let moveset_bytes = moveset
        .as_ref()
        .map_or(0, |m| m.len() * std::mem::size_of::<Move>());
    let peak_logical_bytes = (search.peak_positions + table_entries) * key_bytes + moveset_bytes;

    let result = SolveResult {
        solvable: moveset.is_some(),
        moveset,
        nodes_expanded: search.nodes,
        max_depth: search.max_depth,
        elapsed: search.clock.elapsed(),
        peak_logical_bytes,
        peak_positions: search.peak_positions,
        forced_automoves: search.forced_automoves,
        verdict,
        table_entries,
        table_hits: search.table_hits,
        table_evictions,
        budget_exhausted: search.budget_exhausted,
    };
    (result, search.closed)
}

struct Search<K, F> {
    budget: SolveBudget,
    options: SolveOptions,
    clock: Clock,
    max_time: Option<Duration>,
    nodes: u64,
    max_depth: usize,
    /// On-path ancestors. Never evicted; guarantees termination.
    path: HashSet<K>,
    peak_positions: usize,
    /// Global closed set of proven-winless positions (None when disabled).
    closed: Option<ClosedTable<K>>,
    table_hits: u64,
    forced_automoves: u64,
    found: Option<Vec<Move>>,
    budget_exhausted: bool,
    key_of: F,
}

impl<K, F> Search<K, F>
where
    K: Hash + Eq + Clone,
    F: Fn(&GameState) -> K,
{
    /// Explore `state`. Returns `true` once a win is found (short-circuits). The
    /// state is left unchanged on return (every applied move is undone).
    fn dfs(&mut self, state: &mut GameState, moves: &mut Vec<Move>) -> bool {
        if state.is_won() {
            self.found = Some(moves.clone());
            return true;
        }
        if self.over_budget() {
            self.budget_exhausted = true;
            return false;
        }

        let key = (self.key_of)(state);
        if let Some(table) = &self.closed {
            if table.contains(&key) {
                self.table_hits += 1;
                return false;
            }
        }
        if self.path.contains(&key) {
            return false; // cycle
        }

        self.nodes += 1;
        self.max_depth = self.max_depth.max(moves.len());
        self.path.insert(key.clone());
        self.peak_positions = self.peak_positions.max(self.path.len());

        let won = self.expand(state, moves);

        self.path.remove(&key);
        if !won && !self.budget_exhausted {
            if let Some(table) = &mut self.closed {
                table.insert(key);
            }
        }
        won
    }

    fn expand(&mut self, state: &mut GameState, moves: &mut Vec<Move>) -> bool {
        let candidates = legal_moves(state);

        // Force a provably-safe foundation move when one exists.
        if self.options.safe_automoves {
            if let Some(mv) = safe_move_in(state, &candidates) {
                let undo = apply_undoable(state, mv).expect("safe move is legal");
                self.forced_automoves += 1;
                moves.push(mv);
                let won = self.dfs(state, moves);
                moves.pop();
                undo_move(state, mv, undo);
                return won;
            }
        }

        let prev = moves.last().copied();
        let mut cands = candidates;
        if self.options.empty_column_symmetry {
            cands = apply_empty_column_symmetry(state, cands);
        }
        if self.options.equivalence_pruning {
            cands = apply_equivalence_pruning(state, cands);
        }
        if self.options.move_ordering {
            cands.sort_by_key(|mv| move_priority(state, *mv, prev, self.options.dig_larger_first));
        }

        let mut won = false;
        for mv in cands {
            // No-op structural check on the pre-move state.
            let noop_from = if self.options.no_op_pruning {
                no_op_structural(state, mv)
            } else {
                None
            };
            let undo = match apply_undoable(state, mv) {
                Ok(u) => u,
                Err(_) => continue,
            };
            // Complete the no-op test on the applied state; undo and skip if so.
            if let Some(from) = noop_from {
                if state.tableau[from].is_empty() || !column_has_legal_move(state, from) {
                    undo_move(state, mv, undo);
                    continue;
                }
            }

            moves.push(mv);
            won = self.dfs(state, moves);
            moves.pop();
            undo_move(state, mv, undo);

            if won || self.budget_exhausted {
                break;
            }
        }
        won
    }

    fn over_budget(&self) -> bool {
        if let Some(max) = self.budget.max_nodes {
            if self.nodes >= max {
                return true;
            }
        }
        // On wasm `max_time` is honored as "never" (no monotonic clock), so the
        // node budget alone bounds the search there.
        if let Some(max) = self.max_time {
            if self.nodes & 0x7FF == 0 && self.clock.reached(max) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::ClosedTable;
    use crate::{DrawMode, GameConfig};

    fn draw_one() -> GameConfig {
        GameConfig {
            draw_mode: DrawMode::One,
            redeal_limit: None,
            timed: false,
        }
    }

    /// Seed 2 (draw-one) is solvable in well under a thousand nodes — a fast,
    /// decisive fixture for the reuse/convergence tests.
    fn solvable_root() -> GameState {
        GameState::new_with_seed(2, draw_one())
    }

    #[test]
    fn node_budget_only_returns_well_formed_result() {
        // No time budget: exercises the path that never consults a clock (the
        // only path available on wasm).
        let mut table = ClosedTable::with_capacity(1 << 20);
        let r = solve_reusing(
            &solvable_root(),
            Some(1_000_000),
            SolveOptions::default(),
            &mut table,
        );
        assert_eq!(r.verdict, Verdict::Solvable);
        assert!(r.solvable);
        assert!(r.moveset.is_some());
        assert!(r.nodes_expanded > 0);
    }

    #[test]
    fn reusing_with_full_budget_matches_one_shot() {
        let root = solvable_root();
        let one_shot = solve_state(&root, SolveBudget::default(), SolveOptions::default());
        let mut table = ClosedTable::with_capacity(1 << 20);
        let reused = solve_reusing(&root, Some(10_000_000), SolveOptions::default(), &mut table);
        assert_eq!(reused.verdict, one_shot.verdict);
    }

    #[test]
    fn shared_table_is_consulted_across_calls() {
        // Seed 3 (draw-one) needs hundreds of thousands of nodes, so a small
        // budget leaves the search inconclusive but fills the table. A second
        // call sharing that table must re-hit those proven-winless positions.
        let root = GameState::new_with_seed(3, draw_one());
        let mut table = ClosedTable::with_capacity(1 << 20);
        let first = solve_reusing(&root, Some(2_000), SolveOptions::default(), &mut table);
        assert_eq!(first.verdict, Verdict::Inconclusive);
        assert!(
            table.peak_entries() > 0,
            "first call should record winless positions"
        );
        let second = solve_reusing(&root, Some(2_000), SolveOptions::default(), &mut table);
        assert!(
            second.table_hits > 0,
            "second call should consult the shared table"
        );
    }

    #[test]
    fn repeated_small_budgets_converge_to_the_decisive_verdict() {
        let root = solvable_root();
        let reference = solve_state(&root, SolveBudget::default(), SolveOptions::default());
        assert_eq!(reference.verdict, Verdict::Solvable);

        // Drive many tiny node-bounded slices sharing one table, exactly as the
        // GUI will, and confirm they reach the same decisive verdict.
        let mut table = ClosedTable::with_capacity(1 << 20);
        let mut verdict = Verdict::Inconclusive;
        for _ in 0..5_000 {
            let r = solve_reusing(&root, Some(200), SolveOptions::default(), &mut table);
            if r.verdict != Verdict::Inconclusive {
                verdict = r.verdict;
                break;
            }
        }
        assert_eq!(verdict, Verdict::Solvable);
    }
}
