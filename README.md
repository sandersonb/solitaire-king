# Klondike Solitaire

A Klondike Solitaire implementation in Rust: a pure, deterministic core model
plus a terminal CLI, a graphical (macroquad) front-end that runs natively and in
the browser, and a brute-force + heuristic + transposition-table solver.

## ▶ Play in browser

The GUI is deployed to GitHub Pages on every push to `main`:

**https://sandersonb.github.io/solitairetwo/**

_(Update this link to your actual GitHub Pages URL once Pages is enabled for the
repo under Settings → Pages → "GitHub Actions".)_

## Run locally

```sh
# Graphical (native window)
cargo run --bin klondike-gui -- --seed 42 --draw 3

# Terminal
cargo run --bin klondike -- --seed 42

# Solve a deal
cargo run --bin klondike -- --solve --seed 42
```

### Build the browser version

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --bin klondike-gui
# then serve web/index.html + the .wasm + assets/ (see the Pages workflow)
```

## GUI controls

- **Click** a face-up card to select it (and the run above it), then **click** a
  destination pile to move. Illegal moves are rejected.
- **Click the stock** to deal (recycles when empty).
- **Double-click** a card, or press **Enter**, to auto-move it to its best spot.
- **U** undo · **R** redo · **N** new game · **Esc** deselect.

## Layout

- `src/model/` — the pure, `std`-only game model (rules, scoring, deal).
- `src/solver/` — the automatic solver.
- `src/main.rs`, `src/cli/` — the terminal CLI.
- `src/gui/` — the macroquad GUI (native + WASM).
- `openspec/` — spec-driven change history.
