## MODIFIED Requirements

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

## ADDED Requirements

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
