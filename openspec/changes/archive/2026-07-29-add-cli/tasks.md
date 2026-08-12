## 1. Binary scaffolding and dependencies

- [x] 1.1 Add `[[bin]]` (name `klondike`, `src/main.rs`) to `Cargo.toml` and a binary-only `[dependencies]` section with `crossterm` and `clap` (derive feature); keep the `[lib]` target std-only
- [x] 1.2 Create the `cli` module tree: `src/cli/mod.rs`, `src/cli/session.rs`, `src/cli/render.rs`, `src/cli/input.rs`
- [x] 1.3 Implement `src/main.rs`: parse args, build config, set up/tear down the terminal, and hand off to the session

## 2. Argument parsing and config (cli-shell)

- [x] 2.1 Define a clap args struct: `--cli` (default mode), `-s/--seed <u64>`, `--draw <1|3>`, `--timed`, `--redeal <N>`
- [x] 2.2 Map args to `GameConfig` (`DrawMode`, `redeal_limit`, `timed`) and resolve the seed (random `u64` when `--seed` omitted; capture it for display)
- [x] 2.3 Validate flag values (e.g. reject `--draw 2`) with a clear error + usage; exit without starting a game on invalid input

## 3. Terminal lifecycle and teardown guard

- [x] 3.1 Enter raw mode + alternate screen on startup; restore on exit
- [x] 3.2 Add an RAII/`Drop` guard (and/or panic hook) that always restores the terminal so a panic never leaves it in raw mode

## 4. Board rendering (cli-rendering)

- [x] 4.1 Render a single `Card` with rank + suit glyph (♠♥♦♣) and color (red suits red; black suits contrasting); a concealed marker for face-down; an empty marker for empty slots
- [x] 4.2 Render the tableau: seven columns showing face-down markers then the face-up run, aligned
- [x] 4.3 Render stock (concealed/count), waste (top / last few, draw-mode aware), and the four foundations (suit + top rank or empty), each labeled with its key
- [x] 4.4 Render the status line: seed, move count, current score, elapsed time
- [x] 4.5 Highlight the pending source pile and show a target prompt when a source is selected
- [x] 4.6 Implement the `?` help/legend overlay and a transient message area for rejections/notices

## 5. Input model (cli-input)

- [x] 5.1 Implement the pile key map (`1`–`7` tableau, `8 9 0 -` foundations, `space` stock/waste) shared by source and target
- [x] 5.2 Implement the input state machine (`Idle` / `SourceSelected`) with `Esc` to cancel a pending source
- [x] 5.3 Implement `space` behavior: deal from stock (draw-mode aware); recycle when stock empty (respect redeal limit, message when blocked); select stock/waste as source when a waste card is available, dealing on a second `space`
- [x] 5.4 Implement waste-as-source moves (waste top → chosen tableau/foundation) via validated `Move`s
- [x] 5.5 Implement source→target moves for tableau/foundation sources, applying via `GameState::apply` and rejecting illegal moves with a message
- [x] 5.6 Implement `Enter` auto-assign (waste top when idle; selected source's top otherwise) using `legal_moves`, preferring foundation over tableau; message when no legal destination
- [x] 5.7 Implement forgiving foundation targeting (any foundation key routes a legal card to its suit's foundation)
- [x] 5.8 Implement tableau run handling: apply the unique legal `count`; prompt for a count only when more than one is legal

## 6. Session lifecycle (cli-shell)

- [x] 6.1 Implement the main loop: render → read key → dispatch (move / command) → repeat
- [x] 6.2 Own the wall clock; call `set_elapsed_secs` before scoring/rendering; track and display the move count
- [x] 6.3 Implement undo/redo via `GameState` snapshots (undo stack + redo stack; new move clears redo; restores board, score, counters)
- [x] 6.4 Implement `n` new game (fresh random seed; reset counters/score) and `q` quit (clean terminal restore)
- [x] 6.5 Record the ordered move-history log and output it on quit
- [x] 6.6 Implement the win experience: detect `is_won`, stop the loop, show `final_score` (timed bonus included when `--timed`) and elapsed time

## 7. Verification

- [x] 7.1 Unit-test the pure helpers: pile key mapping, args→`GameConfig` mapping, auto-assign destination choice (foundation-first) over a constructed `GameState`, and run-length resolution
- [x] 7.2 Unit-test undo/redo restores exact prior state and that a new move clears redo
- [x] 7.3 Manual smoke test: `cargo run -- --seed 42` deals, renders with color, plays moves, undo/redo, win/quit restore the terminal
- [x] 7.4 Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` clean
