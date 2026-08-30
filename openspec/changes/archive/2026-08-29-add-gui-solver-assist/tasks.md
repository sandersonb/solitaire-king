## 1. Solver: WebAssembly-safe timing

- [x] 1.1 In `src/solver/search.rs`, remove the unconditional `std::time::Instant` use; gate wall-clock timing behind `cfg(not(target_arch = "wasm32"))` so the arm-deadline and `elapsed` paths compile and run on `wasm32` (report `Duration::ZERO`/no deadline there)
- [x] 1.2 Ensure a node-only budget (`max_time: None`) never touches a clock on any target; the node budget bounds the search everywhere
- [x] 1.3 Test: a search with a node budget and no time budget returns a well-formed result (native), and add a compile/behavior guard so the wasm path can't regress to calling `Instant`

## 2. Solver: reusable transposition table

- [x] 2.1 Re-export the table type: make `mod table` expose `ClosedTable` and add `pub use table::ClosedTable;` in `src/solver/mod.rs`, plus `pub use solver::{ClosedTable, PositionKey}` (as needed) from `src/lib.rs`
- [x] 2.2 Add `solve_reusing(root: &GameState, node_budget, options, table: &mut ClosedTable<PositionKey>) -> SolveResult` in `search.rs`: run the existing search but borrow the caller's table instead of allocating one; leave it populated on return
- [x] 2.3 Refactor `run(...)` so the one-shot `solve_state` and `solve_reusing` share the core search (table owned vs. borrowed); keep `solve`/`solve_state` behavior identical
- [x] 2.4 Tests: (a) `solve_reusing` with a fresh table + full budget matches `solve_state`'s verdict; (b) a second `solve_reusing` sharing the table skips positions the first proved winless (fewer nodes); (c) repeated small-budget calls sharing one table converge to the same decisive verdict as one unbounded call

## 3. GUI solver orchestration

- [x] 3.1 Add `src/gui/solver.rs` with `Status { Unknown, Checking, Solvable, Unwinnable, Inconclusive }` and an `Assist` holding the persistent `ClosedTable<PositionKey>`, a `decided: HashMap<PositionKey, Verdict>`, current status, active-check state, `last_activity`, and dialog/dismissed flags
- [x] 3.2 Implement the per-frame slice pump: while checking, run one node-bounded `solve_reusing` reusing the table, accumulate real solver time via `get_time()`, and finish the check at ~1000 ms (→ Inconclusive) or on a decisive verdict
- [x] 3.3 Implement scheduling: start a check immediately on new deal (task 1 timing makes this wasm-safe); otherwise start a check when idle > ~3 s and status is Unknown/Inconclusive and not known-unwinnable
- [x] 3.4 Implement state-change handling: on move/undo/redo compute the current `PositionKey`, resolve status from `decided` instantly when known (no re-search), else set Unknown + schedule; abandon an in-flight check when the position changes; clear the dismissed-streak flag when leaving Unwinnable
- [x] 3.5 Record decisive results into `decided`; keep exact-byte keys (`KeyStrategy::ExactBytes`) for the check

## 4. Indicator and dialog UI

- [x] 4.1 Render the four-state solvability indicator glyph (solvable / unwinnable / running / uncertain) via the bundled font in a corner, driven by `Status`
- [x] 4.2 Render the unwinnable dialog (dim overlay + panel + **Continue** / **New game** buttons) reusing existing rounded-rect/button drawing; while open it swallows board input
- [x] 4.3 Open the dialog on a proven-unwinnable result only when the streak isn't already dismissed; Continue sets the dismissed flag and closes; New game deals a fresh game and starts a check

## 5. Wire into the game loop

- [x] 5.1 Own an `Assist` in `main.rs`; record activity on applied moves and pointer interaction; call the Assist update + slice pump each frame in `App::Playing`
- [x] 5.2 Reset the Assist (table, decided, flags) and kick an opening check on new game (keyboard N and the on-screen New button, and the dialog's New game)
- [x] 5.3 Route dialog button clicks through the existing pointer/button handling; ensure a move/undo/redo notifies the Assist so status re-resolves

## 6. Validation

- [x] 6.1 `cargo test`, `cargo clippy --all-targets`, `cargo fmt --check`; native + `wasm32-unknown-unknown` builds clean with no new warnings
- [x] 6.2 Manual check against each spec scenario: opening check, idle check, non-blocking play, persistence/reuse, four indicator states, unwinnable dialog + non-nag, undo restoring solvability, known-unwinnable not re-searched (audited at the code level; live in-browser/native GUI smoke test still pending a human)
- [x] 6.3 `openspec validate add-gui-solver-assist --strict`