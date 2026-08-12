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
use std::time::Instant;

use crate::model::rules::{apply_undoable, legal_moves, undo_move};
use crate::solver::classify::{apply_equivalence_pruning, column_has_legal_move, no_op_structural};
use crate::solver::encode::encode;
use crate::solver::heuristics::{apply_empty_column_symmetry, move_priority, safe_move_in};
use crate::solver::table::ClosedTable;
use crate::solver::zobrist::zobrist;
use crate::solver::{KeyStrategy, SolveBudget, SolveOptions, SolveResult, Verdict};
use crate::{GameConfig, GameState, Move};

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
    match options.key {
        KeyStrategy::Zobrist => run(root, budget, options, zobrist),
        KeyStrategy::ExactBytes => run(root, budget, options, encode),
    }
}

/// Run the search with a concrete key function `key_of`.
fn run<K, F>(root: &GameState, budget: SolveBudget, options: SolveOptions, key_of: F) -> SolveResult
where
    K: Hash + Eq + Clone,
    F: Fn(&GameState) -> K,
{
    let start = Instant::now();
    let closed = options
        .transposition_table
        .then(|| ClosedTable::with_capacity(options.max_table_entries));

    let mut search = Search {
        budget,
        options,
        deadline: budget.max_time.map(|d| start + d),
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

    SolveResult {
        solvable: moveset.is_some(),
        moveset,
        nodes_expanded: search.nodes,
        max_depth: search.max_depth,
        elapsed: start.elapsed(),
        peak_logical_bytes,
        peak_positions: search.peak_positions,
        forced_automoves: search.forced_automoves,
        verdict,
        table_entries,
        table_hits: search.table_hits,
        table_evictions,
        budget_exhausted: search.budget_exhausted,
    }
}

struct Search<K, F> {
    budget: SolveBudget,
    options: SolveOptions,
    deadline: Option<Instant>,
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
        if let Some(deadline) = self.deadline {
            if self.nodes & 0x7FF == 0 && Instant::now() >= deadline {
                return true;
            }
        }
        false
    }
}
