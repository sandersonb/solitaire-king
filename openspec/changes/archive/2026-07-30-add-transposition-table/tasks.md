## 1. Options and result surface

- [x] 1.1 Extend `SolveOptions` with `transposition_table: bool` (default true) and `max_table_entries: usize` (default a few million); set both in `Default` and in `baseline()` (table off)
- [x] 1.2 Add a `Verdict { Solvable, Unwinnable, Inconclusive }` enum; add `verdict` plus table stats (`table_entries`, `table_hits`, `table_evictions`) to `SolveResult`; keep `solvable` as a mirror of `verdict == Solvable`
- [x] 1.3 Add `src/solver/table.rs` and wire it into `solver/mod.rs`

## 2. Bounded transposition table (solver-engine)

- [x] 2.1 Implement a fixed-capacity, direct-mapped closed set over `PositionKey`: `contains(&key)`, `insert(key)` (overwrite-on-collision eviction), and hit/eviction/occupancy counters
- [x] 2.2 Unit-test: insert then contains; a colliding insert evicts the prior bucket occupant; occupancy never exceeds capacity; counters update

## 3. Search integration (solver-engine)

- [x] 3.1 Keep the on-path (ancestor) set as the never-evicted cycle guard; ensure termination depends only on it
- [x] 3.2 On node entry, when the table is enabled: skip if the position is in the closed table (count a hit); otherwise proceed
- [x] 3.3 On node exit with no win found, insert the position into the closed table (mark winless); do not insert positions whose subtree found a win or was cut off by the budget
- [x] 3.4 Track whether the search was cut off by a budget limit versus fully exhausted, to drive the verdict
- [x] 3.5 Populate `SolveResult.verdict`: Solvable on a win; Unwinnable when fully exhausted with no win and no budget cut-off; Inconclusive when a budget limit was hit
- [x] 3.6 Fill in table stats (peak entries, hits, evictions) and keep logical-memory accounting driven by the table + on-path set

## 4. Baseline reproduction and validation

- [x] 4.1 Ensure `transposition_table: false` reproduces the archived per-path behavior; `SolveOptions::baseline()` includes it
- [x] 4.2 Extend the differential validator to toggle the table and check verdict agreement on completing positions

## 5. Verification

- [x] 5.1 Integration test: construct a small, genuinely **unwinnable** position; assert verdict == Unwinnable (proven), not Inconclusive, within budget
- [x] 5.2 Integration test: a tiny node budget on a full deal yields Inconclusive (not Unwinnable, not a false win)
- [x] 5.3 Integration test: on a solvable position, table-on and table-off agree on the verdict, and table-on expands no more nodes than table-off
- [x] 5.4 Integration test: eviction soundness — a very small `max_table_entries` yields the same verdict as a large one on a completing position
- [x] 5.5 Re-confirm existing solver + heuristic tests still pass (replayable moveset, safe-automove soundness, cycle termination)

## 6. CLI (solver-cli)

- [x] 6.1 Add `--no-transposition` and `--max-table-entries` flags; fold `--no-transposition` into `--baseline`; map into `SolveOptions`
- [x] 6.2 Update the report to print the three-way verdict (SOLVABLE / UNWINNABLE (proven) / INCONCLUSIVE) and the table stats
- [x] 6.3 Group all clap flags under `help_heading`s by concern ("Game", "Solver search", "Solver heuristics", "Transposition table") so `--help` makes each flag's algorithm/layer clear
- [x] 6.4 Unit-test the flags→`SolveOptions` mapping and the verdict/stat formatting

## 7. Final verification

- [x] 7.1 Manual run: compare `--solve --seed N` (table on) against `--solve --seed N --baseline` across several seeds — the table should solve/prove more within the same budget, and prove at least one deal unwinnable
- [x] 7.2 Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` clean
