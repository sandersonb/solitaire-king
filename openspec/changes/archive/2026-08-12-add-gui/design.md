## Context

The `klondike` library is a pure, deterministic model with a public API (moves, legal-move enumeration, reversible `apply_undoable`/`undo_move`, scoring, pile accessors). A terminal CLI already plays it. This change adds a graphical front-end that reuses the model unchanged, targeting native desktop and the browser (WASM) so anyone can play with no install, deployed automatically to GitHub Pages.

## Goals / Non-Goals

**Goals:**
- A macroquad GUI binary that builds native and WASM (WebGL2) from one codebase.
- Graphical board + click-to-play interaction mirroring the CLI's model; undo/redo, timer, score, config, new game, win.
- Small, browser-friendly bundle; auto-deploy to GitHub Pages.
- Keep the library std-only and the CLI unchanged.

**Non-Goals:**
- Solver in the UI, smooth animation, sound (all deferred).
- Any change to the model or CLI; workspace restructuring.

## Decisions

### macroquad, as a second binary
Add `src/gui/` with a `main.rs` (macroquad `#[macroquad::main]`) and modules (`session`, `render`, `input`, `layout`, `assets`), declared as `[[bin]] name = "klondike-gui"`. `macroquad` goes in `[dependencies]`; only the GUI binary imports it, so the library and CLI keep their current dependency trees (Cargo compiles a dep only for targets that use it). Chosen over bevy for a much smaller WASM bundle and far less code for a 2D card table, and over hand-rolled wgpu for obvious reasons; WebGL2 (macroquad's backend) maximizes browser reach today versus WebGPU.

### Session in the GUI, undo via make/unmake
A `Session` in the GUI binary owns the `GameState`, `GameConfig`, move count, and an undo/redo history. Undo reuses the model's reversible API: the history is a `Vec<(Move, Undo)>`; applying pushes `(mv, apply_undoable(state, mv)?)` and clears redo; undo pops and calls `undo_move`; redo re-applies the move. Because `undo_move` already restores score and recycle count, undo is exact for free. Rationale: no snapshot cloning, and it exercises the reversible-moves capability we just built.

### Cross-target time
`std::time::Instant` is unavailable on `wasm32-unknown-unknown`. The session's clock uses macroquad's `get_time()` (seconds, works native and web); elapsed seconds are pushed into the model via `set_elapsed_secs` before reading `current_score`/`final_score`.

### Rendering: sprites with a procedural fallback
`render` computes a responsive layout (`layout.rs`) from the window size: an upper row (stock, waste, gap, four foundations) and seven overlapping tableau columns below. A card is drawn either from a loaded sprite texture (keyed by rank+suit) or, if sprites are absent, procedurally — a rounded card shape with the rank label and suit glyph (`Rank::label`, `Suit::symbol`), red for hearts/diamonds and dark for clubs/spades. Face-down = a back sprite or a solid patterned rect; empty pile = an outline. The procedural path guarantees a playable game before any art is vendored, and the sprite path is a drop-in upgrade. Assets load via macroquad's async loader with `set_pc_assets_folder("assets")` so the same paths work native and web.

### Card assets
A public-domain 52-card deck plus a back is vendored under `assets/cards/` (filenames like `AS.png`, `10H.png`, `back.png`), with a `assets/NOTICE` citing the Wikimedia PD source. A small fetch script documents how the deck was obtained/regenerated. If fetching art is impractical at implementation time, the procedural renderer ships and the sprite drop-in is documented — the game is playable either way.

### Input as a small state machine
`input.rs` hit-tests the mouse against pile/card rectangles produced by `layout`. State: `selection: Option<Source>` where a source is a tableau card+run, the waste top, or a foundation top. Left-click with no selection selects a source (or, on the stock, deals/recycles); with a selection, a click on a destination attempts the move via `legal_moves` + `apply_undoable` (illegal → message, clear selection). Double-click (two clicks on the same card within a short window) auto-moves; Enter auto-moves the selection or waste top. Keys mirror the CLI where sensible: `u` undo, `r` redo, `n` new game, `Esc` deselect. Auto-move chooses among legal moves preferring a foundation, matching the CLI.

### Config on native vs web
Native launch parses `--seed`/`--draw`/`--timed` (reuse clap). The browser has no argv; the web build starts with defaults (draw-three, untimed, random seed) and the player uses new-game (and a draw-mode toggle on new game) in-app. Reading URL query params on web is a possible later enhancement, out of scope now.

### Splash screen and build metadata
The app starts in a `Splash` state before `Playing`. The splash centers the `king-logo` texture with the title (`Klondike`), version (`env!("CARGO_PKG_VERSION")`), build date, and author, and dismisses on any click/key or after ~2–3 seconds (tracked with `get_time()`), transitioning to `Playing`. Build date is captured by a `build.rs` that emits `cargo:rustc-env=BUILD_DATE=YYYY-MM-DD` — computed from `SystemTime::now()` via a tiny pure-std civil-date conversion (no new dependency), read in-app with `env!("BUILD_DATE")`. Author comes from `env!("CARGO_PKG_AUTHORS")` (set the package `authors` field). The logo is loaded once as a reusable texture and MAY be drawn elsewhere (e.g. on the win banner); if the logo asset is missing, the splash degrades gracefully to text only. Rationale: a lightweight app-state enum keeps the splash out of the play loop, and a compile-time build date avoids any runtime/date dependency and works identically native and web.

### Distribution
`web/index.html` is a static shim that loads macroquad's JS bootstrap and the `.wasm`. A `.github/workflows/deploy-pages.yml` workflow, on push to `main`: installs the `wasm32-unknown-unknown` target, `cargo build --release --target wasm32-unknown-unknown --bin klondike-gui`, assembles `dist/` (wasm + `index.html` + `assets/`), and deploys with the GitHub Pages actions. The README gets a "▶ Play in browser" link to the Pages URL.

## Risks / Trade-offs

- **Vendoring real card art may be impractical in the build environment** → The procedural renderer is the primary path and guarantees a playable game; sprites are an optional, documented drop-in. No hard dependency on fetching binaries.
- **WASM quirks (time, blocking, file access)** → Use macroquad's `get_time()` and async asset loader (never `std::Instant`/`std::fs`); this is macroquad's supported cross-target path.
- **Bundle size / load time on Pages** → macroquad keeps WASM small (~2–5 MB); assets are a modest set of PNGs. Acceptable for a static Pages site.
- **macroquad isn't WebGPU** → Intentional: WebGL2 maximizes "any modern browser" reach today; a WebGPU path is not needed for a card game.

## Open Questions

- Exact PD deck to vendor and its sprite naming scheme — will pick a clean English-pattern set at implementation time; the renderer's fallback removes the risk of blocking on it.
- Web config surface (URL params vs in-app toggles) — starting with in-app new-game + draw-mode toggle; URL params are a later nicety.
