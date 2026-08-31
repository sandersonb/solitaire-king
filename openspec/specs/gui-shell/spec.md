# gui-shell Specification

## Purpose

The graphical application shell: the GUI binary and its dual native/WebAssembly
targets, the startup splash screen, game configuration and seeding, session
tracking (moves, score, timer), undo/redo, and new-game and win handling.

## Requirements

### Requirement: GUI binary and dual target
The application SHALL provide a GUI binary that opens a window and plays Klondike graphically, and SHALL build for both native desktop and WebAssembly (WebGL2) from the same source so it runs in a modern browser.

#### Scenario: Native launch
- **WHEN** the GUI binary is run natively
- **THEN** a window opens showing a dealt game ready to play

#### Scenario: Browser launch
- **WHEN** the WASM build is loaded in a modern browser
- **THEN** the same game renders and is playable with the mouse

### Requirement: Startup splash screen
On launch, the GUI SHALL show a splash screen displaying the `king-logo` artwork, the application title and version, the build date, and the author, before entering play. The splash SHALL be dismissible by the player (a click or key press) and SHALL also advance on its own after a short delay. The build date SHALL be captured at compile time; the author SHALL come from package metadata.

#### Scenario: Splash shown at launch
- **WHEN** the GUI starts
- **THEN** a splash screen appears showing the logo, title/version, build date, and author

#### Scenario: Splash dismisses into the game
- **WHEN** the player clicks or presses a key on the splash, or the short delay elapses
- **THEN** the splash is dismissed and the dealt game is shown

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
The GUI SHALL track and display the number of moves made, the current score, and the elapsed play time, and SHALL display the seed (as the pronounceable seed string) subject to a show/hide-seed setting. The clock SHALL work on both native and web (it SHALL NOT rely on a timer unavailable in the browser), and elapsed time SHALL be supplied to the model before the score is computed.

#### Scenario: Metrics update during play
- **WHEN** the player makes a move
- **THEN** the move count increases and the displayed score reflects the model's score

#### Scenario: Timer advances
- **WHEN** time passes during play
- **THEN** the displayed elapsed time increases (in both native and browser builds)

#### Scenario: Seed shown as a readable string
- **WHEN** the show-seed setting is on and the session metrics are displayed
- **THEN** the seed appears as the pronounceable seed string

#### Scenario: Seed hidden when the setting is off
- **WHEN** the show-seed setting is off
- **THEN** the seed is hidden from the status area

### Requirement: Undo and redo
The GUI SHALL support undoing the most recent move and redoing an undone move, restoring the exact prior state (board, score, counters). Undo SHALL be implemented with the model's reversible-move API. Starting a new move after an undo SHALL clear the redo history.

#### Scenario: Undo restores the prior state
- **WHEN** the player makes a move and then undoes it
- **THEN** the board, score, and move count return to exactly their values before that move

#### Scenario: Redo re-applies an undone move
- **WHEN** the player undoes a move and then redoes it
- **THEN** the state returns to after that move

### Requirement: New game and win handling
The GUI SHALL let the player start a fresh game (a new random seed) at any time, and SHALL, on a win reached by play, display a win indication with the final score (including the timed bonus when timed) and the elapsed time. A win reached by auto-solve SHALL instead be indicated as auto-solved rather than presented as a scored win.

#### Scenario: New game re-deals
- **WHEN** the player starts a new game
- **THEN** a fresh deal appears and the move count and score reset

#### Scenario: Win is shown
- **WHEN** all four foundations are completed by play
- **THEN** a win banner appears showing the final score and elapsed time

#### Scenario: Auto-solved finish is distinct
- **WHEN** the game is completed by auto-solve
- **THEN** the finish is indicated as auto-solved, not shown as a scored win

### Requirement: Touch session controls
The GUI SHALL let the player perform the core session actions on a touch device with no keyboard: undo the most recent move and start a new game via on-screen controls. Redo MAY remain keyboard-only. These controls SHALL drive the same session logic as the existing keyboard commands.

#### Scenario: Undo without a keyboard
- **WHEN** the player uses the on-screen undo control after a move
- **THEN** the last move is undone, identically to the keyboard undo

#### Scenario: New game without a keyboard
- **WHEN** the player uses the on-screen new-game control
- **THEN** a fresh game is dealt with a new seed, resetting the move count and score

### Requirement: Settings
The GUI SHALL provide a settings dialog with: the draw mode (one or three) applied
to the next new game (the current game is unchanged); a toggle for the background
solver; and a show/hide toggle for the seed. Settings SHALL take effect within the
session; persistence across launches is not required.

#### Scenario: Draw-mode setting applies to the next game
- **WHEN** the player changes the draw mode in settings and then starts a new game
- **THEN** the new deal uses the chosen draw mode while the prior game was unaffected

#### Scenario: Solver toggle takes effect
- **WHEN** the player disables the background solver in settings
- **THEN** background checks stop

#### Scenario: Seed visibility toggle takes effect
- **WHEN** the player toggles show/hide seed
- **THEN** the status area shows or hides the seed accordingly

### Requirement: Auto-solve session semantics
While the game is being auto-solved and after it finishes by auto-solve, the session
SHALL NOT accrue score and SHALL zero the timer, so an auto-solved deal is not
recorded as a scored, timed win.

#### Scenario: Auto-solve does not score
- **WHEN** the game is completed by auto-solve
- **THEN** the score is not counted for the finish and the elapsed timer reads zero

### Requirement: Asset loading progress
Before the game is playable, the GUI SHALL load its assets in a way that lets it
show progress (a spinner or progress indicator) rather than a blank screen, and SHALL
proceed to play once loading completes. Missing optional assets SHALL NOT block
loading (consistent with the existing procedural fallbacks).

#### Scenario: Progress shown while assets load
- **WHEN** the GUI is loading its card and font assets
- **THEN** a loading indicator is shown until loading completes, after which play proceeds
