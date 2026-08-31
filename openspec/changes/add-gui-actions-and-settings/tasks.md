## 1. Loading screen

- [x] 1.1 In `web/index.html`, add a full-screen `#loading` spinner (CSS) behind the canvas (`z-index: 0`; canvas `z-index: 1`) so it shows during the WASM download and is covered once the first opaque frame paints
- [x] 1.2 Make `Assets::load` stream: load textures/font one at a time, and every few assets draw a progress bar (built-in font + rectangles) and `next_frame().await`; tolerate missing optional assets
- [x] 1.3 Enter the splash only after streaming load completes; confirm native + wasm still start cleanly

## 2. Declutter + pointer hold support

- [x] 2.1 Remove the keyboard/drag help-text line from `draw_status` in `src/gui/render.rs`; keep the metrics line
- [x] 2.2 Re-add a `down` (held-this-frame) field to `Pointer` in `input.rs` and populate it from mouse/touch (needed for hold detection)
- [x] 2.3 Fix touch hit-testing on high-DPI/iOS: `touches()` reports physical pixels while `mouse_position()`/`screen_*` are logical, so divide touch coords by `screen_dpi_scale()` in `read_pointer` (touch was landing off by the DPI factor — down/right of the finger)

## 3. Control bar: buttons + indicator rect

- [x] 3.1 Extend `layout.rs` `ButtonId` to `{ UndoRedo, New, Settings }` and lay out three bar buttons plus reserve the indicator slot
- [x] 3.2 Add `render::indicator_rect(layout) -> Rect` and draw the indicator there; render the three buttons with pressed feedback; the indicator shows a muted "off" badge when the solver is disabled
- [x] 3.3 Implement Undo/Redo button: tap → `undo()`, press-and-hold past ~0.4 s → one `redo()` (track press time + fired flag; cancel if the pointer leaves the button)

## 4. Overlay modality

- [x] 4.1 Add a GUI-owned `Overlay { None, Solver, Settings }`; establish per-frame input priority: unwinnable dialog → overlay → board (only call `handle_input` when nothing is open)
- [x] 4.2 Route control-bar/indicator clicks in the board path: Settings button opens the Settings overlay; the indicator opens the Solver overlay when status is not Checking

## 5. Solver overlay + retained line

- [x] 5.1 In `solver.rs`, add `solution: Option<(PositionKey, Vec<Move>)>` set when a check returns Solvable (store `moveset`); expose `solution_len()` / `solution_for(key)` and an `enabled` flag with `set_enabled`
- [x] 5.2 Render the Solver overlay per status (`render.rs`): solvable → "solution exists in N moves" + Auto-solve + Close; unwinnable → undo-may-help + New game + Close; uncertain/unknown → play-more + Close; disabled → solver-off note
- [x] 5.3 Wire overlay buttons: Auto-solve starts auto-solve (task 6); New game deals fresh; Close dismisses; clicking outside dismisses

## 6. Auto-solve + paced playback

- [x] 6.1 In `anim.rs`, pace the queue: `enqueue_moves` records a start time; add `take_next(now)` that yields the next move only when `now >= next_at`, advancing `next_at` by ~0.5 s; update the loop drain to use it
- [x] 6.2 In `session.rs`, add `auto_solving`/`auto_solved` flags with begin/finish handling; while auto-solving the timer is held at zero and the finish is marked auto-solved (score/time suppressed in display, not in the model)
- [x] 6.3 Trigger auto-solve (from the overlay button and `Shift+A`, only when `solution_for(current)` is `Some`): enqueue the retained line, set the session auto-solving; on reaching the won state, mark auto-solved
- [x] 6.4 Status/win rendering: show score as "—" and time as 0 while auto-solving/auto-solved; the win banner reads "Auto-solved" instead of a scored win

## 7. Settings

- [x] 7.1 Add a GUI-owned `Settings { draw: DrawMode, solver_enabled: bool, show_seed: bool }` seeded from the launch config
- [x] 7.2 Render the Settings overlay (draw 1/3 choice, solver-enable toggle, seed show/hide) and handle row toggles
- [x] 7.3 Apply effects: `new_game` uses `settings.draw` for the next deal; `solver_enabled` gates `Assist` (via `set_enabled`); `show_seed` passed to `draw_status`

## 8. Validation

- [x] 8.1 `cargo test`, `cargo clippy --all-targets`, `cargo fmt --check`; native + `wasm32-unknown-unknown` builds clean with no new warnings
- [x] 8.2 Manual check against each spec scenario: loading indicator, no help line, indicator opens overlays, solvable N-moves + Auto-solve + Shift+A, paced auto-solve to win with suppressed score/timer + "Auto-solved" banner, tap-undo/hold-redo, settings (draw next-game, solver toggle, seed show/hide) — audited at the code level; live native/browser GUI smoke test still pending a human
- [x] 8.3 `openspec validate add-gui-actions-and-settings --strict`