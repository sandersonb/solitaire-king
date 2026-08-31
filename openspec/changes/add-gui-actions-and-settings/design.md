## Context

See `proposal.md` — Why. Relevant current state:

- The control bar (`layout.rs`) has two buttons (Undo, New); `render.rs` draws them
  and a status line that includes a keyboard-command help line. The solvability
  indicator (`render::solver_indicator`) is draw-only — it has no rect the input
  layer knows about.
- Input is a unified `Pointer { x, y, pressed, released }` (the `down`/held field was
  removed as unused). Modal input already exists for the unwinnable dialog
  (`handle_dialog`, gated in the loop).
- `Assist` (`solver.rs`) tracks a `Status` and a `decided: HashMap<PositionKey,
  Verdict>` but does **not** keep the winning moveset. `SolveResult.moveset` has the
  line when solvable.
- `Animator` (`anim.rs`) has an unused `enqueue_moves`/`next_queued` queue; the loop
  drains one queued move per idle frame (instantly).
- `Session` derives score from state (`current_score`/`final_score`) and is fed
  `elapsed` each frame; `undo`/`redo` stacks exist.
- `Assets::load()` loads ~107 textures + the font up front, before any frame renders,
  so nothing can be drawn during load. `web/index.html` shows a bare canvas.

## Goals / Non-Goals

**Goals:** the six proposal items, cohesive with the existing modality and control
bar; no regressions to drag/animation/solver behavior.

**Non-Goals:**
- No settings persistence across launches (in-session only).
- No change to the model, rules, scoring, or the solver engine.
- No solver *hint* highlighting (only the full auto-solve).
- No true progress % for the WASM download itself (the page spinner is
  indeterminate); only the in-app asset phase shows discrete progress.

## Decisions

### D1: Loading — page spinner + in-app progress, no JS interop
`web/index.html` gains a full-screen `#loading` spinner **behind** the canvas
(`z-index: 0`; canvas `z-index: 1`). The canvas is transparent until the first Rust
frame clears it to an opaque background, which then covers the spinner — so no JS
hook is needed to hide it. For the asset phase, `Assets::load` becomes streaming: it
loads one asset at a time and, every few, draws a progress bar (built-in font, plain
rectangles) and `next_frame().await`. The opening splash follows once loading is done.
- *Alternative:* a Rust→JS `hide_loader()` import. Rejected — macroquad's import
  object is fixed; the behind-canvas trick is simpler and reliable.

### D2: Indicator as a button + a single modal layer
Add `render::indicator_rect(layout) -> Rect` (shared by draw + hit-test). Introduce a
GUI-owned `Overlay { None, Solver, Settings }`. The unwinnable auto-dialog stays
`Assist`-driven. Input priority each frame: **unwinnable dialog → overlay → board**.
`handle_input` is only called when no dialog/overlay is open; dedicated handlers run
otherwise. The control-bar buttons and the indicator are hit-tested in the board path
(they are the overlay entry points).

### D3: Solver overlay content by status
Clicking the indicator (when not `Checking`) opens the Solver overlay:
- `Solvable` → "A solution exists in N moves" + **Auto-solve** + Close. N and the
  line come from D4.
- `Unwinnable` → "This deal can't be won; undoing may reopen a solution." + New game +
  Close. (The auto-dialog remains the first-time, non-nagging warning; this is the
  on-demand view.)
- `Inconclusive`/`Unknown` → "Play more moves to determine solvability." + Close.
- `Checking` → indicator inert (no overlay).
When the background solver is **disabled** (D6), the indicator shows a muted "off"
badge and its overlay explains the solver is off.

### D4: Retain the winning line
`Assist` gains `solution: Option<(PositionKey, Vec<Move>)>`, set whenever a check
returns `Solvable` (store `r.moveset`). `solution_len()` and `solution_for(current)`
back the overlay and auto-solve. Auto-solve/Shift+A are offered only when
`solution_for(current_key)` is `Some` — i.e., the line was found for the position on
screen. (A decided-solvable position revisited via undo without a retained line simply
shows Solvable; a fresh idle check re-derives its line.)

### D5: Auto-solve via the paced animation queue
Auto-solve enqueues the retained line into the `Animator` and puts the session in
auto-solve mode. `Animator` gains a cadence: `enqueue_moves` records a start time and
`take_next(now)` yields the next move only when `now >= next_at`, setting
`next_at = now + PLAY_SECS` (~0.5 s). The loop's drain uses `take_next` and, as today,
only when no drag and no animation is in flight, so each move animates (~0.14 s) then
waits out the remaining cadence.
- **Session semantics (D5a):** `Session` gains `auto_solving`/`auto_solved` flags.
  While `auto_solving`, the loop feeds `set_elapsed(0)` (frozen timer) and the status
  shows score as "—"; reaching the won state sets `auto_solved`, and the win banner
  reads "Auto-solved" instead of a scored win. Moves still apply normally (state must
  advance to the win); only the *display/accounting* of score+time is suppressed, per
  the request.

### D6: Settings
A GUI-owned `Settings { draw: DrawMode, solver_enabled: bool, show_seed: bool }`,
initialized from the launch config. The Settings button opens the Settings overlay;
clicking a row toggles it. Effects:
- **draw:** used by `new_game` for the *next* deal; the current game is untouched.
- **solver_enabled:** `Assist::set_enabled(false)` stops checks and clears status to an
  "off" indicator; `true` re-evaluates the current position.
- **show_seed:** passed to `draw_status`; hidden means the seed is omitted from the
  status line.

### D7: Undo button — tap vs hold
Re-add `down` to `Pointer` (held-this-frame), needed for hold detection. Track
`undo_press: Option<f64>` (press time over the Undo button) and `undo_fired: bool`.
While held over the button: once `now - press > HOLD_SECS` (~0.4 s) and not yet fired,
do one `redo()` and set `undo_fired`. On release: if not fired, `undo()`. Pointer
leaving the button cancels. Keyboard U/R still call undo/redo directly.

### D8: Declutter
Remove the help-text line from `draw_status`; keep the metrics line (seed subject to
D6). Keyboard handlers (`U`/`R`/`N`/`Enter`, plus new `Shift+A`) remain.

## Risks / Trade-offs

- **Auto-solve from a state without a retained line** → offered only when the line is
  retained for the current position (D4); avoids a blocking re-solve on wasm.
- **Score/time truly unchanged vs. suppressed** → the model still recomputes score as
  state advances; we suppress its *display* and zero elapsed while auto-solving. This
  matches "do not increment score and zero the timer" without touching the model.
- **Hold-vs-tap ambiguity** on touch → a conservative ~0.4 s threshold; a hold that
  wanders off the button cancels rather than misfiring.
- **Streaming loader frame pacing** → progress redraw every few assets (not every one)
  keeps load fast; missing optional assets are skipped (existing fallbacks hold).
- **Page spinner never hidden if the first frame never paints** (fatal wasm error) →
  acceptable: that already means the app failed to start; the spinner is the least of
  it.

## Migration Plan

1. `web/index.html` spinner + `Assets` streaming loader with an in-app progress screen.
2. Declutter status text; re-add `Pointer.down`.
3. Control bar: Undo/Redo (tap/hold) + New + Settings buttons; `indicator_rect`.
4. `Overlay` modality + input priority; Solver overlay (D3) + Settings overlay (D6).
5. `Assist`: retain line (D4), `set_enabled` (D6), auto-solve trigger; Shift+A.
6. `Animator` paced playback (D5); `Session` auto-solve semantics (D5a); wire the loop.
7. Verify native + wasm builds, tests, clippy, fmt.
Rollback is per-item and GUI-local; none of it touches the library.

## Open Questions

- The cadence (~0.5 s) and hold threshold (~0.4 s) are tuning values, adjustable
  during implementation without changing the specs or task breakdown.
