## 1. Options and result surface

- [x] 1.1 Extend `SolveOptions` with `safe_automoves`, `move_ordering`, `empty_column_symmetry` (default true) and `dig_larger_first` (default true); keep `no_op_pruning` true, `equivalence_pruning` false
- [x] 1.2 Add `forced_automoves: u64` (or similar) to `SolveResult` and initialize it
- [x] 1.3 Add a `src/solver/heuristics.rs` module and wire it into `solver/mod.rs`

## 2. Safe foundation auto-moves (solver-engine)

- [x] 2.1 Implement `safe_foundation_move(state) -> Option<Move>`: a foundation move whose card rank ≤ 2, or both opposite-color foundations ≥ rank−1
- [x] 2.2 In the search, when `safe_automoves` is on and a safe move exists, play it as the node's only child (forced) and increment `forced_automoves`
- [x] 2.3 Unit-test the safe predicate: aces/2s always safe; rank-r safe iff both opposite-color foundations ≥ r−1 (incl. the uneven-foundations case); unsafe case not flagged

## 3. Move ordering (solver-engine)

- [x] 3.1 Implement an ordering key over candidate `Move`s: reveals-face-down (tie-break by digging direction) > productive builds/waste plays > stock draw (penalize if previous move was a draw) > foundation→tableau last
- [x] 3.2 Thread the previous move into the node step so the consecutive-draw penalty applies
- [x] 3.3 Apply ordering only when `move_ordering` is on; ensure it reorders without removing any legal move
- [x] 3.4 Unit-test ordering: revealing before drawing; consecutive-draw deprioritized; foundation→tableau last; larger-vs-smaller digging direction

## 4. Empty-column symmetry (solver-engine)

- [x] 4.1 When `empty_column_symmetry` is on and ≥2 columns are empty, keep only one King-to-empty destination among the candidates
- [x] 4.2 Unit-test: with multiple empty columns, only one King-to-empty move is generated; with one empty column, behavior is unchanged

## 5. Search integration

- [x] 5.1 Restructure the per-node step: forced safe move → else generate legal moves, apply symmetry + no-op/equivalence filters, then order, then iterate (clone/apply, per-path cycle check, recurse, stop at first win)
- [x] 5.2 Ensure `SolveOptions::default()` yields the heuristic solver and disabling all new flags reproduces the baseline
- [x] 5.3 Extend the differential validator so each heuristic can be toggled and verdict-preservation checked

## 6. Verification

- [x] 6.1 Integration test: heuristics preserve the verdict — for a batch of seeds, solvable(heuristics on) == solvable(all new heuristics off) under the same budget
- [x] 6.2 Integration test: on the near-won position, heuristics-on solves with fewer nodes than heuristics-off (forced auto-moves + ordering reduce work)
- [x] 6.3 Integration test: find at least one seed the baseline reports inconclusive but the heuristic solver solves within the same budget (demonstrating the improvement)
- [x] 6.4 Re-confirm existing solver tests (no-op soundness, cycle termination, replayable moveset) still pass

## 7. CLI (solver-cli)

- [x] 7.1 Add `--solve` flags to disable each heuristic and set digging direction, plus a `--baseline` convenience flag that disables all heuristics at once; map them into `SolveOptions`
- [x] 7.2 Include a heuristic statistic (forced auto-moves played) in the solve report
- [x] 7.3 Unit-test the flags→`SolveOptions` mapping and the extended report formatting

## 8. Final verification

- [x] 8.1 Manual run: `cargo run -- --solve --seed <N>` finds a win on a winnable deal where the baseline (`--solve` with heuristics disabled) does not, within the same budget
- [x] 8.2 Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` clean
