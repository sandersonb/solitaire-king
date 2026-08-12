## 1. Solver module scaffolding

- [x] 1.1 Create the `src/solver/` module tree (`mod.rs`, `encode.rs`, `classify.rs`, `search.rs`) and declare it in `lib.rs`
- [x] 1.2 Define the public surface in `mod.rs`: `SolveBudget`, `SolveOptions` (pruning toggles), `SolveResult`, and `solve(seed, config, budget, options) -> SolveResult`; re-export from the crate root

## 2. Position encoding (solver-state-encoding)

- [x] 2.1 Implement a 1-byte card codec (rank + suit + face-up) and a `PositionKey(Vec<u8>)` newtype with `Hash`/`Eq`/`Clone`
- [x] 2.2 Encode tableau columns (length-prefixed, in order, with face-up flags), stock, and waste from the public accessors
- [x] 2.3 Encode foundations canonically by suit (top rank per suit in fixed suit order)
- [x] 2.4 Append recycles-remaining only when the redeal limit is bounded; omit under unlimited
- [x] 2.5 Unit-test: determinism, positional-only (score/time excluded), foundation-slot canonicalization, and bounded-vs-unlimited redeal behavior

## 3. Move classification (solver-engine)

- [x] 3.1 Implement the no-op predicate: tableau→tableau, reveals no face-down card, interchangeable host (empty→empty, or same-rank-same-color host), and exposed source card has no legal move
- [x] 3.2 Ensure draws, recycles, foundation-targeting/foundation-source moves, and card-revealing moves are never classified as no-ops
- [x] 3.3 Implement the equivalence predicate (interchangeable destinations for a freshly available card) as a separate, off-by-default rule
- [x] 3.4 Unit-test no-op classification against the canonical examples (King empty→empty; red 2 between two black 3s; a revealing move; stock/foundation moves)

## 4. Search engine (solver-engine)

- [x] 4.1 Implement DFS: generate `legal_moves`, filter via active pruning rules, clone+apply each successor, recurse, backtrack
- [x] 4.2 Implement per-path cycle detection using a `HashSet<PositionKey>` pushed on descent and popped on backtrack
- [x] 4.3 Implement `SolveBudget` enforcement (max nodes, max time) with checks at each node
- [x] 4.4 Detect wins via `is_won`; stop at the first win and retain its moveset
- [x] 4.5 Track statistics: nodes expanded, max depth, elapsed time, and peak logical memory (path-set positions × encoded bytes + retained moveset + working state)
- [x] 4.6 Populate `SolveResult`, distinguishing solved from budget-exhausted (inconclusive, never "unwinnable")

## 5. Differential validation (solver-engine)

- [x] 5.1 Implement a validation helper that runs `solve` with and without a chosen optional rule over a set of seeds and reports verdict agreement/discrepancies
- [x] 5.2 Test-suite: run the no-op rule through the validator over a batch of seeds and assert solvable-verdict agreement (soundness guard)

## 6. Solver verification

- [x] 6.1 Integration test: construct a near-won position, assert `solve` finds a win and the returned moveset replays to a won state (every move legal, final state won)
- [x] 6.2 Integration test: a trivially unsolvable/blocked small position under a tiny budget reports inconclusive (budget-exhausted), not a false win
- [x] 6.3 Test cycle detection prunes a constructed revisit, and that no-op pruning reduces node counts without changing the solvable verdict

## 7. CLI integration (solver-cli)

- [x] 7.1 Add `--solve` and solver budget flags (`--max-nodes` default 10,000,000, `--max-time` default 15s) plus a pruning toggle to the clap args
- [x] 7.2 In `main`, branch on `--solve`: build the config, run `solve`, and skip interactive setup
- [x] 7.3 Print the report: solvable?, winning moveset (when solved), elapsed time, peak logical memory; state "inconclusive" on budget exhaustion
- [x] 7.4 Unit-test the args→budget/options mapping and the report formatting for solved vs inconclusive

## 8. Final verification

- [x] 8.1 Manual run: `cargo run -- --solve --seed <N> --max-time 5` prints a coherent report
- [x] 8.2 Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` clean
