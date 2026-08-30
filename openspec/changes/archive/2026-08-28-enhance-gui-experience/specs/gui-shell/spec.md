## MODIFIED Requirements

### Requirement: Game configuration
The GUI SHALL play with a configurable draw mode (one or three) and an optional timed mode, and SHALL deal from a seed. On native, these MAY be supplied as launch arguments; the seed argument SHALL accept either a pronounceable seed string or a raw `u64`. A random seed SHALL be used when none is given, and the seed SHALL be shown (as the pronounceable seed string) so a deal is reproducible.

#### Scenario: Reproducible deal from a seed
- **WHEN** the GUI is launched with the same seed and configuration twice
- **THEN** both sessions show the identical starting board

#### Scenario: Seed argument accepts either form
- **WHEN** the GUI is launched with a seed given as the pronounceable string or as the equivalent raw `u64`
- **THEN** both produce the identical deal

#### Scenario: Draw mode is honored
- **WHEN** the game is configured for draw-one or draw-three
- **THEN** dealing from the stock turns that many cards

### Requirement: Session tracking
The GUI SHALL track and display the seed (as the pronounceable seed string), the number of moves made, the current score, and the elapsed play time. The clock SHALL work on both native and web (it SHALL NOT rely on a timer unavailable in the browser), and elapsed time SHALL be supplied to the model before the score is computed.

#### Scenario: Metrics update during play
- **WHEN** the player makes a move
- **THEN** the move count increases and the displayed score reflects the model's score

#### Scenario: Timer advances
- **WHEN** time passes during play
- **THEN** the displayed elapsed time increases (in both native and browser builds)

#### Scenario: Seed shown as a readable string
- **WHEN** the session metrics are displayed
- **THEN** the seed appears as the pronounceable seed string

## ADDED Requirements

### Requirement: Touch session controls
The GUI SHALL let the player perform the core session actions on a touch device with no keyboard: undo the most recent move and start a new game via on-screen controls. Redo MAY remain keyboard-only. These controls SHALL drive the same session logic as the existing keyboard commands.

#### Scenario: Undo without a keyboard
- **WHEN** the player uses the on-screen undo control after a move
- **THEN** the last move is undone, identically to the keyboard undo

#### Scenario: New game without a keyboard
- **WHEN** the player uses the on-screen new-game control
- **THEN** a fresh game is dealt with a new seed, resetting the move count and score
