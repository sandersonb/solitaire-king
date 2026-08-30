## Why

The project has a capable brute-force solver, but the GUI never uses it. Players
get no feedback on whether the deal they're playing is still winnable, and can
grind on a position that became unwinnable moves ago. Surfacing the solver in the
GUI — running it unobtrusively in the background and showing a live solvability
indicator — turns a hidden engine into a genuinely useful assist, and gives us a
reason to make the solver work on the web build (where it currently can't run).

## What Changes

- **Background solvability checking in the GUI.** After a new deal, and again
  whenever play has been idle for ~3 seconds, the GUI evaluates the current
  position with the solver, capped at ~1000 ms of solver work. Results are often
  **inconclusive** and that is expected.
- **Non-blocking, cross-platform execution.** The solve runs cooperatively in
  small per-frame slices so the game never freezes — the same mechanism on native
  and WebAssembly (the deployed build is single-threaded).
- **Persistent search knowledge.** The solver's transposition table is preserved
  in memory across solves and across moves, so later checks (and re-checks after
  undo) reuse prior work and are fast.
- **Solvability indicator.** An on-screen icon shows the current status: a
  solution exists, proven unwinnable, a check is running (hourglass), or
  uncertain/inconclusive (?).
- **Unwinnable dialog.** When the current position is proven unwinnable, a dialog
  says so and offers **Continue** or **New game**. It does not nag: once
  acknowledged, it stays dismissed until solvability returns.
- **Undo/redo aware.** "Unwinnable" pins to a specific position, not the whole
  game. Undoing back to a winnable position clears the unwinnable state and
  resumes checking; re-reaching a known-unwinnable position is recognized
  instantly from the preserved table without re-searching.
- **BREAKING (internal solver API):** the solver gains a WebAssembly-safe timing
  path and a caller-owned reusable transposition table entry point. Existing
  `solve`/`solve_state` behavior is preserved.

## Capabilities

### New Capabilities
- `gui-solver`: The GUI's background solvability assist — when checks run, how the
  solver is driven cooperatively without blocking the frame, the four-state
  solvability indicator, the unwinnable dialog with continue/new-game, and the
  per-position (undo-aware, non-nagging) status model.

### Modified Capabilities
- `solver-engine`: Add platform-independent (WebAssembly-safe) time budgeting so
  the engine runs on the web target; add a caller-owned, reusable transposition
  table so repeated node-bounded searches share proven-winless knowledge and make
  monotonic progress across slices and across positions.

## Impact

- **Solver library (`src/solver/`):** `search.rs` timing made `wasm32`-safe (no
  unconditional `std::time::Instant`); a new entry point that accepts a
  caller-provided `ClosedTable` and a node budget and returns partial progress;
  `ClosedTable` (and a fixed key type for persistence) exposed via `lib.rs`.
- **GUI (`src/gui/`):** a new solver-orchestration module (idle detection, slice
  pump, status state machine, table ownership); `render.rs` for the indicator icon
  and the unwinnable dialog; `input.rs`/`main.rs` for dialog buttons and resetting
  status on move/undo/redo/new-game.
- **No changes** to the deal, rules, scoring, or the CLI solver's behavior. The
  reused table is sound because positions are keyed by the complete encoded state;
  exact-byte keys are used for the GUI check so a proven-unwinnable claim cannot be
  a hash collision.
- **Performance:** solver work is bounded per frame and per check (~1000 ms total),
  so the target frame rate is maintained on native and web.
