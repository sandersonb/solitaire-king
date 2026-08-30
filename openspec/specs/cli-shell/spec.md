# cli-shell Specification

## Purpose

Defines the CLI application shell for the Klondike game: the binary entry point and mode selection, command-line configuration, the interactive game loop, elapsed-time and move tracking, new-game/quit/win handling, undo/redo, and the move-history log.

## Requirements

### Requirement: Binary entry and mode selection
The application SHALL provide a binary that, when run, launches the interactive CLI game. A `--cli` flag SHALL select the interactive CLI mode, and this SHALL be the default mode when no mode flag is given, so running the binary with no arguments starts a new game immediately.

#### Scenario: Run with no arguments
- **WHEN** the binary is run with no arguments
- **THEN** a new game is dealt and the interactive CLI starts

#### Scenario: Explicit --cli flag
- **WHEN** the binary is run with `--cli`
- **THEN** the interactive CLI starts, identically to running with no mode flag

### Requirement: Command-line configuration
The binary SHALL accept flags that configure the game: `-s`/`--seed <SEED>` to set the deal seed, where `<SEED>` is either a pronounceable seed string or a raw `u64` (a random seed SHALL be used when omitted); `--draw <1|3>` to choose the draw mode (default 3), `--timed` to enable timed scoring, and `--redeal <N>` to cap stock recycles (unlimited when omitted). These SHALL be translated into a `GameConfig` and seed used to create the game. Invalid flag values — including a `--seed` that is neither a valid seed string nor a `u64` — SHALL produce a clear error and usage message instead of starting a game.

#### Scenario: Seed controls the deal
- **WHEN** the binary is run twice with the same `--seed` and the same other flags
- **THEN** both runs deal the identical starting board

#### Scenario: Seed accepts either form
- **WHEN** `--seed` is given as a pronounceable seed string or as the equivalent raw `u64`
- **THEN** both deal the identical starting board

#### Scenario: Draw mode selection
- **WHEN** the binary is run with `--draw 1`
- **THEN** the game uses draw-one mode
- **WHEN** the binary is run with `--draw 3` or without `--draw`
- **THEN** the game uses draw-three mode

#### Scenario: Invalid flag value
- **WHEN** the binary is run with an unsupported value such as `--draw 2` or a `--seed` that is neither a valid seed string nor a `u64`
- **THEN** it prints an error with usage and exits without starting a game

### Requirement: Interactive game loop
The CLI SHALL run a loop that renders the current board, waits for player input, applies the resulting action, and repeats. The loop SHALL continue until the player quits or the game reaches a won state.

#### Scenario: Render-input-apply cycle
- **WHEN** the loop is running and the player makes a legal move
- **THEN** the move is applied and the board is re-rendered to reflect the new state

#### Scenario: Loop ends on win
- **WHEN** applying a move results in a won game
- **THEN** the loop stops and the win experience is shown

### Requirement: Elapsed time and move tracking
The CLI SHALL own a wall clock, tracking elapsed play time, and SHALL supply it to the model (via `set_elapsed_secs`) before computing or displaying the score. The CLI SHALL count the number of moves the player has applied this session.

#### Scenario: Elapsed time advances
- **WHEN** time passes during play
- **THEN** the displayed elapsed time increases and, in timed mode, is reflected in the score

#### Scenario: Move count increments
- **WHEN** the player applies a legal move
- **THEN** the displayed move count increases by one

### Requirement: New game, quit, and win handling
The CLI SHALL let the player start a fresh game (a new random seed) with `n`, quit with `q`, and SHALL, on a win, display the final score — including the timed bonus when timed mode is enabled — and the elapsed time. On quit, the session summary SHALL show the deal's seed as the pronounceable seed string so the deal can be replayed.

#### Scenario: Quit exits cleanly
- **WHEN** the player presses `q`
- **THEN** the CLI restores the terminal and exits without error

#### Scenario: Quit summary shows a shareable seed
- **WHEN** the player quits
- **THEN** the summary shows the deal's seed as the pronounceable seed string

#### Scenario: New game re-deals
- **WHEN** the player presses `n`
- **THEN** a fresh game is dealt with a new seed, resetting the move count and score

#### Scenario: Win shows final score
- **WHEN** the game is won
- **THEN** the final score and elapsed time are displayed, using `final_score` so the timed bonus is included when timed mode is on

### Requirement: Undo and redo
The CLI SHALL support undoing the most recent move and redoing an undone move, by snapshotting `GameState`. Undo SHALL restore the exact prior state (board, score, counters); starting a new move after an undo SHALL clear the redo history.

#### Scenario: Undo restores prior state
- **WHEN** the player applies a move and then presses `u` (undo)
- **THEN** the board, score, and move count return to exactly their values before that move

#### Scenario: Redo re-applies an undone move
- **WHEN** the player undoes a move and then requests redo
- **THEN** the undone move is re-applied and the state returns to after that move

### Requirement: Move-history log
The CLI SHALL record the ordered sequence of moves applied during the session and SHALL make it available (for example, printed on quit), providing a foundation for future replay and solver features.

#### Scenario: History captures applied moves
- **WHEN** the player has applied several moves and then quits
- **THEN** the session can output the ordered list of those moves
