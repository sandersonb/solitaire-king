## Why

The `klondike` core model is complete but has no way to play a game. This change adds the first playable surface: an interactive terminal CLI that deals a game, renders the board with color and Unicode, takes keystroke input, enforces the rules, and runs until the player wins or quits. It turns the library into something a human can actually sit down and play.

## What Changes

- Add a **binary target** (`src/main.rs`) so `cargo run` launches the game. A `--cli` flag selects the interactive CLI (the default mode for now), reserving room for other front-ends later.
- Add **command-line flags**: `-s`/`--seed <u64>` (omit for a random seed), `--draw <1|3>` (default 3), `--timed`, and `--redeal <N>` (default unlimited). These map onto `GameConfig`.
- Add an **interactive game loop**: render the board → read input → apply a legal move → repeat, until the game is won, the player quits, or (future) the game is detected unwinnable.
- Add a **board renderer** using UTF-8 suit glyphs (♠♥♦♣) and ANSI color (red vs. black suits, highlighted selections, dimmed face-down cards), showing the tableau, stock, waste, all four foundations, plus a status line with the **seed, move count, score, and elapsed time**.
- Add a **keystroke input model**: a source key then a target key build a move — `1`–`7` = tableau columns, `8 9 0 -` = foundations, `space` = the stock/waste. `space` deals from the stock (recycles when empty); `Enter` auto-assigns a card to its best legal destination. **Illegal moves are rejected** (never applied), with brief on-screen feedback.
- Add **quality-of-life features**: `u` undo / redo (via `GameState` snapshots), `n` new game, `?` help/legend overlay, and an in-session **move-history log** (printable on quit) that lays groundwork for future replay/solver hooks.
- Add a **win experience**: on completion, show the final score (including the timed bonus when `--timed`) and elapsed time.

Non-goals (deferred): the automatic solver and unwinnable-state detection, save/load or replay file formats, non-CLI UIs, mouse input, and any change to the core rules/model.

## Capabilities

### New Capabilities

- `cli-shell`: The binary entry point, argument/flag parsing, config construction, and the game-session lifecycle — bootstrapping from a seed, the main loop, new game, quit, win handling, undo/redo, and the move-history log.
- `cli-rendering`: The terminal board renderer — layout of tableau/stock/waste/foundations and the status line (seed, moves, score, time), using color and Unicode, plus the help/legend overlay.
- `cli-input`: The keystroke interaction model — the source→target keymap, stock dealing/recycling, `Enter` auto-assignment, run-count handling, and rejection of illegal moves.

### Modified Capabilities

<!-- None. The core model capabilities (klondike-domain-model, klondike-rules-engine, klondike-scoring) are consumed unchanged. -->

## Impact

- **New binary target**: `src/main.rs` plus a `cli` module tree (`cli/mod.rs`, rendering, input, session). The existing `klondike` library is unchanged and still `no-deps`.
- **New dependencies (binary only)**: `crossterm` (raw single-key input + cross-platform color) and `clap` (argument parsing). Declared so they do not affect the library's dependency-free status.
- **Cargo.toml**: add `[[bin]]` and a `[dependencies]` section for the binary; the library target stays std-only.
- **Consumes the existing public API**: `GameState::new_with_seed`, `GameConfig`/`DrawMode`, `legal_moves`, `GameState::apply`, `is_won`, `current_score`/`final_score`, `set_elapsed_secs`, and the pile accessors. No model changes required.
