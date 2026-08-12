## Context

The `klondike` library is a pure, deterministic, I/O-free core (domain model, rules engine, Windows Standard scoring). It exposes `GameState::new_with_seed`, `GameConfig`/`DrawMode`, `legal_moves`, `GameState::apply`, `is_won`, `current_score`/`final_score`, `set_elapsed_secs`, and pile accessors. This change builds the first human-facing surface on top of it: an interactive terminal CLI. All game rules stay in the library; the CLI is presentation, input, and session orchestration only.

## Goals / Non-Goals

**Goals:**
- A runnable binary (`cargo run`) that deals and plays a full game in the terminal.
- Clear, colorful board rendering with Unicode suit glyphs and a metrics status line (seed, moves, score, time).
- A fast keystroke input model (source→target, deal, auto-assign) that only ever applies legal moves.
- Quality-of-life: undo/redo, new game, help overlay, move-history log, and config via flags.
- Keep the library dependency-free; confine `crossterm`/`clap` to the binary.

**Non-Goals:**
- The automatic solver and unwinnable detection (future change).
- Save/load or replay file formats (we only capture in-memory history now).
- Non-CLI UIs, mouse input, and any change to the core model.

## Decisions

### Binary + module layout
Add `src/main.rs` (entry, arg parsing, terminal setup/teardown) and a `cli` module: `cli/mod.rs`, `cli/session.rs` (game lifecycle, loop, undo/redo, history), `cli/render.rs` (board painting), and `cli/input.rs` (key handling + move construction). Rationale: mirrors the library's clean module separation and keeps rendering, input, and orchestration independently testable. `Cargo.toml` gains a `[[bin]]` and a binary-only `[dependencies]`; the `[lib]` target stays std-only.

### Dependencies: `crossterm` + `clap`
`crossterm` for raw-mode single-keypress reading and cross-platform styling (color, dim, clear); `clap` (derive) for `--cli`, `-s/--seed`, `--draw`, `--timed`, `--redeal`. Rationale: single-keystroke input effectively requires raw mode; hand-rolling termios would be Unix-only and error-prone, and crossterm is the de-facto standard. clap gives correct `--help`/usage and validation cheaply. Alternatives: raw termios + ANSI (rejected: portability/maintenance), hand-rolled arg parsing (rejected now that config flags exist — clap's validation earns its keep). The library core remains dependency-free, preserving that property from the model change.

### Input as an explicit state machine
Model input as a small state machine with an optional "pending source": `Idle` and `SourceSelected(pile)`. Keys resolve as:
- Idle: a pile key selects a source (→ SourceSelected); `Enter` auto-plays the waste top; `space` engages the stock/waste (deal immediately when there is nothing in the waste to act on, otherwise select it as source); `u`/`n`/`?`/`q`/`Esc` are commands.
- SourceSelected: a pile key attempts source→target; `Enter` auto-plays the source's top; a second `space` (when source is stock/waste) deals; `Esc` cancels.

Rationale: this cleanly reconciles the two roles of `space` (deal vs. select the waste) without clobbering an existing waste card, and makes "illegal moves are not permitted" trivial — every constructed move is checked against `legal_moves`/`apply` before taking effect. The lone-`space`-deals-when-waste-empty shortcut keeps the common early-game action to one key. **Open for confirmation at apply time** (see Open Questions).

### Moves are validated, never assumed
The CLI constructs a candidate `Move` and applies it via `GameState::apply`, which returns `Result<(), MoveError>`; on `Err` the state is untouched and the CLI shows a transient message. For enumerable choices (auto-assign, run length, forgiving foundation routing) the CLI consults `legal_moves(&state)` and filters. Rationale: the library is the single source of truth for legality; the CLI never re-implements rules. Auto-assign picks from legal moves preferring foundations, then tableau.

### Run-length handling
For a tableau→tableau move, compute the legal `TableauToTableau { from, to, count }` values from `legal_moves`. In practice the destination's top card uniquely determines the run bottom, so there is normally exactly one legal `count`; apply it directly. Only when more than one legal count exists (defensive) does the CLI prompt for a number. Rationale: matches the chosen "prompt only when ambiguous" behavior while keeping the common path zero-friction.

### Undo/redo via state snapshots
`GameState` is `Clone`, so the session keeps an undo stack of prior `GameState` snapshots (and mirrors move count / history), plus a redo stack. Applying a new move clears redo. Rationale: snapshots are simple and correct given small state; no need for move inversion. Trade-off: memory grows with history length — acceptable for a single interactive game; could switch to move-inversion later if needed.

### Time ownership
The CLI owns a `std::time::Instant` started at deal; each render it computes elapsed seconds and calls `set_elapsed_secs` before reading `current_score`/`final_score`. Rationale: the model is deliberately clock-free for determinism; the front-end supplies time. In untimed mode elapsed time is displayed but does not affect score.

### Rendering approach
Full-screen repaint each cycle (clear + draw) using crossterm styling: red for hearts/diamonds, bright/white for clubs/spades, dim for face-down (a concealed glyph such as `🂠`/`▒`), reverse-video highlight for the pending source. Foundations show their suit and top rank or an empty marker; the waste shows up to the last few cards (draw-3 aware); the stock shows a count/concealed marker. Status line renders seed, moves, score, elapsed time; a `?` overlay lists keys. Rationale: full repaint is simplest and flicker is negligible at human input rates.

## Risks / Trade-offs

- **Raw-mode terminal left in a bad state on panic** → Install a teardown guard (RAII/`Drop` or a panic hook) that always disables raw mode and restores the screen before exit.
- **Unicode/emoji or 256-color not supported by every terminal** → Use widely supported suit glyphs and basic ANSI colors; keep a plain fallback path if color is unavailable. Avoid exotic emoji that render inconsistently.
- **Windows terminal quirks** → crossterm abstracts this; verify color/raw mode on the primary target (macOS) and rely on crossterm's cross-platform support otherwise.
- **`space` dual meaning confuses players** → The state machine and the waste-empty shortcut minimize surprise; the `?` help overlay documents it; behavior is confirmable at apply time.
- **Undo memory growth** → Bounded in practice by a single game's move count; revisit with move-inversion only if it matters.

## Open Questions

_All resolved (confirmed 2026-07-29):_

- **`space` semantics — CONFIRMED.** Use the state-machine resolution: a lone `space` deals only when the waste has no card to act on; once the waste has a top card, `space` selects the stock/waste as the source and a second `space` deals (so an existing waste card is never dealt over). No extra keys.
- **Random seed — CONFIRMED.** When `--seed` is omitted, derive a `u64` from OS/time entropy at startup and display it, so the game stays reproducible and shareable.
- **Help overlay — CONFIRMED.** Ship a compact legend for now (not a full key reference).
