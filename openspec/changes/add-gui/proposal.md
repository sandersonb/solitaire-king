## Why

The game is fully playable in the terminal, but a graphical version makes it approachable to anyone and — crucially — playable in a browser with no install. This change adds a 2D GUI on top of the existing `klondike` model, targeting both native desktop and WebAssembly so it runs in any modern browser, and wires up automatic GitHub Pages deployment so there's a "▶ Play in browser" link straight from the repo.

## What Changes

- Add a **macroquad-based GUI binary** (`klondike-gui`) that renders the board and plays a full game with the mouse. macroquad builds to **native and WASM (WebGL2)** from one codebase, giving broad browser support (Safari/Chrome/Firefox) with a small (~2–5 MB) bundle. The existing CLI binary and the std-only library are untouched.
- **Graphical board**: the seven tableau columns, stock, waste, and four foundations, with face-up cards showing rank + suit, face-down cards as a card back, empty piles as placeholders, and a status area (seed, moves, score, elapsed time). Cards render from **PNG sprites when present** (a public-domain deck vendored into `assets/`), with a **procedural fallback** (rank/suit drawn on a card shape) so the app is always playable.
- **Click-based interaction** mirroring the CLI: click a face-up card to select it and the run above it; click a destination pile to move (illegal moves rejected with brief feedback); click the stock to deal (recycle when empty); double-click a card or press Enter to **auto-move** it to its best legal spot. Plus keys for new game and undo.
- **Session features from the CLI in graphical form**: draw-1/3 and timed config, a live timer and score, move count, **undo/redo** (reusing the model's `apply_undoable`/`undo_move`), new game, and a win banner.
- **Startup splash screen**: on launch, a brief splash shows the **`king-logo` artwork**, the app title/version, the **build date**, and the **author**, then dismisses (on click/key or after a moment) into the game. The logo is loaded as a reusable asset and MAY appear elsewhere (e.g. the win banner).
- **Browser distribution**: a WASM build config, a static `index.html` shim, and a **GitHub Actions workflow** that builds the WASM and deploys to **GitHub Pages** on push to `main`, with a README "Play in browser" link.

Non-goals (explicitly deferred): the **solver** in the UI, **smooth animation** (cards snap to position this iteration), **sound**, and any change to the model or CLI.

## Capabilities

### New Capabilities

- `gui-shell`: The GUI binary entry, native + WASM targets, asset loading, the main loop, and the game session — config (draw mode, timed, seed), timer/score, move count, undo/redo, new game, and win handling.
- `gui-rendering`: The graphical board — layout of tableau/stock/waste/foundations, card sprites with a procedural fallback, the status area, selection highlighting, and transient messages.
- `gui-input`: The mouse/keyboard interaction model — select→move, deal, and auto-move, only ever applying legal moves.
- `gui-distribution`: The browser build and deployment — WASM build, static page, and the GitHub Pages CI workflow.

### Modified Capabilities

<!-- None. The model and CLI capabilities are consumed unchanged. -->

## Impact

- **New binary target**: `src/gui/` (main + modules) declared as a second `[[bin]]` (`klondike-gui`); `macroquad` added to `[dependencies]` (used only by the GUI binary — the library and CLI don't import it and stay as they are).
- **New assets**: `assets/` (a PD card deck + back, plus a NOTICE with the source; the existing `assets/king-logo.png` is used for the splash) and a web `index.html` shim.
- **Build metadata**: a `build.rs` captures the build date into a compile-time env var for the splash; the author is read from package metadata.
- **New CI**: `.github/workflows/deploy-pages.yml` building the WASM and publishing to GitHub Pages; a README "Play in browser" link.
- **Consumes the existing public API**: `GameState`, `GameConfig`/`DrawMode`, `legal_moves`, `apply_undoable`/`undo_move`, `is_won`, `current_score`/`final_score`, `set_elapsed_secs`, `Suit::symbol`/`Rank::label`, and the pile accessors. No model changes.
- **Cross-target note**: the session clock uses macroquad's time (not `std::time::Instant`, which is unavailable on `wasm32`).
