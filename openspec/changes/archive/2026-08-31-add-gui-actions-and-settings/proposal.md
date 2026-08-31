## Why

The GUI now has drag-and-drop, a control bar, and a background solvability check,
but the solver result is passive (just an icon), there's no way to act on it, the
on-screen text still advertises keyboard shortcuts, and there's no settings or
loading feedback. This change turns the solver into something the player can use
(see the solution, auto-solve), tidies the controls onto buttons, and adds the
missing shell affordances (settings, loading screen).

## What Changes

- **Declutter the status text.** Remove the on-screen help line
  ("drag to move · dbl-click/Enter · U undo · R redo · N new"). Undo/redo/new are
  driven by buttons; the keyboard shortcuts keep working silently for power users.
- **Solver status becomes an interactive button.** Clicking (or tapping) the
  solvability indicator opens a small state-dependent overlay:
  - **Solution exists** → "A solution exists in N moves" plus an **Auto-solve**
    button; **Shift+A** also auto-solves (only when a solution exists).
  - **No solution** → a note that the deal can't be won, but undoing may reopen
    a solution.
  - **Uncertain** → a note suggesting more moves may reveal solvability.
  - **Working** → no action (a check is in progress).
- **Auto-solve.** Play the found solution to completion with a ~500 ms delay
  between moves, animated. Auto-solving does **not** accrue score and **zeroes
  the timer** — the finish is shown as auto-solved, not a scored win.
- **Undo/redo on one button.** The **Undo** button undoes on a tap and redoes on
  a press-and-hold (a single redo per hold).
- **Settings dialog** with: draw mode (one/three, applied to the next new game),
  a toggle for the background solver, and show/hide the seed in the status line.
- **Loading screen.** A spinner while the WebAssembly downloads (in the page), and
  an in-app progress bar while card/font assets load, so the first view isn't a
  blank canvas.

## Capabilities

### New Capabilities
_None — this extends existing GUI capabilities._

### Modified Capabilities
- `gui-rendering`: Remove the help-text line; render the indicator as a button;
  render the solver action overlays, the Settings dialog, and the in-app asset
  loading progress screen.
- `gui-input`: Indicator button opens the solver overlay; overlay/settings button
  interactions; **Shift+A** auto-solve; the Undo button's tap-undo / hold-redo
  behavior; opening Settings. Keyboard shortcuts remain but are no longer shown.
- `gui-shell`: A Settings model (draw mode for next new game, background-solver
  enable, seed visibility); auto-solve session semantics (no score, zeroed timer,
  auto-solved finish); staged asset loading that yields progress.
- `gui-solver`: Retain the winning moveset and its length when a position is
  solvable; state-dependent indicator actions; the auto-solve trigger; honor the
  background-solver enable toggle (no checks when disabled).
- `gui-animation`: Auto-solve playback plays queued moves at a fixed cadence
  (~500 ms apart) rather than immediately.
- `gui-distribution`: The web page shows a loading spinner until the app's first
  frame paints.

## Impact

- **GUI code (`src/gui/`):** `render.rs` (remove help text; indicator-as-button;
  overlays; settings dialog; loading screen), `input.rs`/`main.rs` (indicator/
  overlay/settings hit-testing, Shift+A, Undo tap-vs-hold, activity), `solver.rs`
  (store moveset + count; enable toggle; auto-solve start), `anim.rs` (timed
  playback cadence), `session.rs` (auto-solve mode: freeze timer, suppress score),
  `assets.rs` (staged/step loader), and a small `Settings` holder.
- **Web (`web/index.html`):** a CSS spinner behind the canvas, covered once the
  first frame paints (no JS interop needed).
- **No CLI changes** and no changes to the model, rules, scoring, or the solver
  engine (auto-solve replays existing legal moves; the "no score" behavior is a
  GUI display/accounting choice).
