## Why

The GUI is functional but reads as a prototype: the built-in bitmap font looks
poor at high DPI, there is no touch/mobile story, moves are made with a
click-select/click-target model gated behind a yellow highlight, and deals are
identified by a bare `u64` seed that nobody can remember or share by voice. This
change makes the GUI pleasant on a phone and a high-resolution desktop, replaces
selection with direct manipulation (drag-and-drop with animation), and gives
every surface a memorable seed string.

## What Changes

- **Better text rendering.** Bundle a legible open-license (OFL) sans font and
  render all GUI text (splash, status line, on-screen buttons, procedural card
  fallback) through it, so text is crisp at high DPI.
- **Mobile play.** Detect touch / small-or-portrait viewports and switch to a
  mobile layout with an on-screen button bar (**Undo**, **New**; no Redo).
  Support touch input for dragging and buttons. Load a **separate,
  higher-legibility mobile card image set** (bigger corner indices) on mobile so
  cards read on small screens; fall back to the desktop art (then procedural)
  when absent.
- **Memorable seeds (CLI + GUI).** Encode the `u64` deal seed as a pronounceable
  **proquint** string (e.g. `lusab-babad-gutih-tugad`) in the library, used for
  display everywhere and accepted as input. The `--seed` argument and any seed
  entry SHALL accept either a proquint string or a raw `u64` (back-compat).
- **Drag-and-drop with animation.** Replace click-select + click-target (and the
  yellow selection highlight) with press-drag-release. The dragged card/run
  follows the pointer (lifted above the finger and enlarged on touch). Releasing
  near a legal destination snaps via **invisible drop zones**: the card does not
  need to land precisely — it animates (tweens) into the pile's resting position.
  Illegal releases animate back to the origin. Double-click/tap and Enter still
  auto-move. A reusable **card-animation/tween subsystem** drives these moves and
  is designed so future automated play (solver "gestures") can enqueue and play
  back animated moves.

## Capabilities

### New Capabilities
- `seed-encoding`: A reversible, pronounceable encoding of the 64-bit deal seed
  (proquint) plus a decoder that also accepts a raw `u64`; the shared contract
  every surface (CLI, GUI, solver report) uses to display and parse seeds.
- `gui-animation`: A card animation/tween subsystem that moves cards between
  screen positions over time, used by drag-drop snapping and available for
  future automated move playback.

### Modified Capabilities
- `gui-input`: Direct-manipulation drag-and-drop and touch input replace
  click-select/click-target; add on-screen button controls; drop-zone based
  release resolution.
- `gui-rendering`: Bundled font for all text; removal of the selection
  highlight in favor of a dragged-card visual and optional legal-target hints;
  rendering the on-screen button bar; mobile card art with fallback; responsive
  mobile/portrait layout.
- `gui-shell`: Mobile/touch controls (undo, new) as first-class session
  actions; seed shown as a proquint string; seed configurable as proquint or
  `u64`.
- `cli-shell`: `--seed` accepts a proquint string or a raw `u64`; the session
  summary prints the proquint seed.
- `cli-rendering`: The status line shows the seed as a proquint string.
- `solver-cli`: The solver report shows the seed as a proquint string.

## Impact

- **New library API:** `klondike::seed` (or similar) with `encode(u64) -> String`
  and `decode(&str) -> Option<u64>`; re-exported from `lib.rs`. Pure `std`, so it
  compiles for both native and `wasm32`.
- **GUI code:** `src/gui/` — new animation module; `input.rs` reworked for
  drag/drop/touch; `render.rs` for font, buttons, mobile art, dragged card;
  `layout.rs` for mobile/portrait layout and button-bar/drop-zone rects;
  `assets.rs` for font + mobile card set; `main.rs` loop for drag state and
  animation ticking.
- **CLI code:** `src/main.rs` (arg parsing accepts proquint), `src/cli/render.rs`
  and `src/cli/session.rs` and `src/cli/solve.rs` (seed display).
- **Assets:** a bundled `.ttf`/`.otf` font and a `cards-mobile/` image set under
  `assets/`; the GitHub Pages deploy workflow must ship both (assets are served
  under `dist/assets/`).
- **No changes** to the deal algorithm, RNG, rules, scoring, or solver search —
  seeds remain fully reproducible; proquint is purely a presentation/parsing layer
  over the same `u64`.
