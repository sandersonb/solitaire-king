## 1. Seed encoding (library, shared)

- [x] 1.1 Add `src/model/seed.rs` with proquint `encode(u64) -> String` (four `-`-joined quints)
- [x] 1.2 Add `decode(&str) -> Option<u64>`: case-insensitive, strip `-`/whitespace, valid-quint path first, then raw-`u64` fallback, else `None`
- [x] 1.3 Register the module in `src/model/mod.rs` and re-export `klondike::seed::{encode, decode}` from `src/lib.rs`
- [x] 1.4 Unit tests: round-trip across many `u64` (incl. `0` and `u64::MAX`), raw-`u64` fallback, mixed case / stray separators, and rejection of invalid input

## 2. CLI + solver seed presentation

- [x] 2.1 Change `--seed` in `src/main.rs` from `u64` to `String`; decode via `seed::decode`, and on failure print an error + usage and exit (like the existing invalid-flag path)
- [x] 2.2 Show `seed::encode(seed)` in the CLI status line (`src/cli/render.rs`) and the quit summary (`src/main.rs` `print_summary`)
- [x] 2.3 Show `seed::encode(seed)` in the solver report (`src/cli/solve.rs`)
- [x] 2.4 Update/extend affected tests (e.g. `args_parse_smoke`, solve report assertions) for the string seed

## 3. GUI text rendering (font)

- [x] 3.1 Add an OFL sans font at `assets/fonts/ui.ttf` and its license entry in `assets/NOTICE`
- [x] 3.2 Load it in `src/gui/assets.rs` into `Assets { font: Option<Font> }` (tolerate absence)
- [x] 3.3 Add a `text(...)`/`measure(...)` helper in `src/gui/render.rs` that uses `draw_text_ex`/`measure_text` with the font when present, else the built-in font
- [x] 3.4 Route all GUI text (splash, status line, win banner, procedural card fallback, buttons) through the helper

## 4. GUI seed display + `--seed` parsing

- [x] 4.1 Parse the GUI native `--seed` arg as a seed string via `seed::decode` (accept proquint or `u64`)
- [x] 4.2 Display the seed as `seed::encode(...)` in the GUI status line

## 5. Responsive layout + on-screen controls

- [x] 5.1 Extend `Layout::compute` to take viewport size/aspect (+ touch flag) and select a desktop vs. mobile/portrait profile that reserves a control-bar band and keeps 7 columns
- [x] 5.2 Add button rects (`Undo`, `New`) and a `drop_zones()` helper (padded per-pile rects incl. origin) to `Layout`
- [x] 5.3 Render the control bar in `src/gui/render.rs` (buttons visible, not overlapping piles, pressed-state feedback)
- [x] 5.4 Wire button activation (click/tap) to `session.undo()` and new-game; keep keyboard commands working

## 6. Mobile card art

- [x] 6.1 Add `assets/cards-mobile/` set (higher-legibility indices) and its NOTICE entry
- [x] 6.2 Load it in `assets.rs` into `Assets { cards_mobile }` (optional)
- [x] 6.3 Make `draw_card` resolve mobile → desktop → procedural based on the layout profile

## 7. Drag-and-drop + touch input

- [x] 7.1 Add a `Pointer` abstraction unifying mouse and `touches()` (press/move/release + position), preferring touch when active to avoid duplicate web events
- [x] 7.2 Replace the `Option<Source>` selection with a `Drag` state (source, run, grab offset, pointer, origin rects); remove the yellow selection highlight
- [x] 7.3 Implement press (hit-test source; stock deals/recycles), move (run follows pointer; lifted+enlarged on touch), and release (nearest drop zone → `resolve` → move or reject)
- [x] 7.4 Preserve double-click/tap and Enter auto-move and the rejected-move message

## 8. Card animation subsystem

- [x] 8.1 Add `src/gui/anim.rs` with a tween list ticked by elapsed time (from/to/dur/ease)
- [x] 8.2 On legal release, apply the move then enqueue a snap animation from release point to the pile's resting rect; on illegal release enqueue a return-to-origin animation
- [x] 8.3 Draw in-flight cards over the board at the interpolated position; keep state changes ahead of animation so input/scoring never block
- [x] 8.4 Add an `enqueue_moves(&[Move])` playback entry point (unused by UI now) to support future automated/solver move playback

## 9. Distribution + docs

- [x] 9.1 Update `.github/workflows/deploy-pages.yml` to ship `assets/fonts/` and `assets/cards-mobile/` under `dist/assets/` (and prune build-time `cards-svg/`)
- [x] 9.2 Verify the web build loads the font, mobile cards, and touch drag in a browser; update CLAUDE.md notes for the new asset dirs (wasm build + dist assembly verified locally; live in-browser smoke test still pending a human)
- [x] 9.3 Run `cargo test`, `cargo clippy`, and a native + wasm build; confirm no new warnings and a playable game

## 10. Validation

- [x] 10.1 Run `openspec validate enhance-gui-experience --strict` and resolve any issues
- [x] 10.2 Manual check against each modified/added spec scenario (seed forms, mobile layout, drag-drop, animation, buttons, font fallback)
