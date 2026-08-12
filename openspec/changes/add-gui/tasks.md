## 1. GUI binary scaffolding

- [x] 1.1 Add `macroquad` to `[dependencies]` and a `[[bin]] name = "klondike-gui"` (path `src/gui/main.rs`); confirm the library and CLI binary build unaffected
- [x] 1.2 Create the `src/gui/` module tree: `main.rs`, `session.rs`, `layout.rs`, `render.rs`, `input.rs`, `assets.rs`
- [x] 1.3 Stand up a minimal macroquad main loop (window, clear, poll, present) that opens a window natively

## 2. Session (gui-shell)

- [x] 2.1 Implement `Session`: owns `GameState`, `GameConfig`, move count, seed, and undo/redo history
- [x] 2.2 Apply moves via `apply_undoable` (push `(mv, undo)`, clear redo, increment moves); undo via `undo_move`; redo by re-applying
- [x] 2.3 Drive the clock with macroquad `get_time()` (not `std::Instant`); push elapsed seconds via `set_elapsed_secs` before reading score
- [x] 2.4 Implement new game (fresh random seed, reset counters) and win detection (`is_won`) with final score
- [x] 2.5 Native arg parsing (`--seed`/`--draw`/`--timed`) via clap; web build uses defaults + in-app new game
- [x] 2.6 Unit-test the pure session logic: apply/undo/redo restores state and counters; new game resets
- [x] 2.7 Add `build.rs` emitting `cargo:rustc-env=BUILD_DATE=YYYY-MM-DD` (pure-std epoch→civil date, no new dep); set the package `authors` field; add an app-state enum (`Splash`/`Playing`) with dismiss-on-click/key or short timeout via `get_time()`

## 3. Layout and rendering (gui-rendering)

- [x] 3.1 Implement `layout.rs`: responsive positions/rects for stock, waste, foundations, and the seven overlapping tableau columns from the window size; expose hit-test rectangles
- [x] 3.2 Implement `assets.rs`: async-load card sprites and back into a lookup keyed by rank+suit, plus the `king-logo` texture; tolerate missing assets
- [x] 3.3 Implement `render.rs`: draw each card from its sprite, or procedurally (card shape + `Rank::label`/`Suit::symbol`, red vs black) when sprites are absent; face-down back; empty-pile placeholder
- [x] 3.4 Render the status area (seed, moves, score, elapsed time), selection highlight, and a transient message line
- [x] 3.5 Render the win banner (final score + elapsed time) on a won game
- [x] 3.6 Render the splash screen (centered `king-logo`, title + `CARGO_PKG_VERSION`, `BUILD_DATE`, author), degrading to text if the logo is missing; reuse the logo on the win banner

## 4. Input (gui-input)

- [x] 4.1 Hit-test mouse clicks against layout rects; identify the clicked pile and (for tableau) the specific card + run above it
- [x] 4.2 Selection state machine: click selects a source (tableau run / waste top / foundation top); click empty space or same source deselects
- [x] 4.3 Destination click: build and apply the legal move (run for tableau sources) via `legal_moves`; reject illegal with a message
- [x] 4.4 Stock click: deal (draw-mode aware) or recycle when empty (respect redeal limit)
- [x] 4.5 Auto-move: double-click a card, or Enter on the selection/waste top → best legal destination (foundation-first); message when none
- [x] 4.6 Keyboard commands: `u` undo, `r` redo, `n` new game, `Esc` deselect

## 5. Card assets

- [x] 5.1 Vendor a public-domain 52-card deck + back into `assets/cards/` with an `assets/NOTICE` citing the Wikimedia PD source (document the fetch/regeneration script); if impractical, ship the procedural renderer and document the sprite drop-in

## 6. Browser build + distribution (gui-distribution)

- [x] 6.1 Add `web/index.html` (macroquad JS bootstrap + `.wasm` loader) and verify `cargo build --target wasm32-unknown-unknown --bin klondike-gui` produces a runnable artifact
- [x] 6.2 Add `.github/workflows/deploy-pages.yml`: on push to `main`, build the WASM, assemble `dist/` (wasm + index.html + assets), and deploy to GitHub Pages
- [x] 6.3 Add a "▶ Play in browser" link to the README pointing at the Pages URL

## 7. Verification

- [x] 7.1 Manual: `cargo run --bin klondike-gui` deals, renders the board, plays a scripted sequence (select→move, deal, auto-move, undo, new game), and shows a win
- [ ] 7.2 Manual: build the WASM target and load `web/index.html` locally to confirm it renders and is playable in a browser
- [x] 7.3 Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` clean (native targets)
