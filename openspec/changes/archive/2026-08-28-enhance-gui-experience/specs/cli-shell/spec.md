## MODIFIED Requirements

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
