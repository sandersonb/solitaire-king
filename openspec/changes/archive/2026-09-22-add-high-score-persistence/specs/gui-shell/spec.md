## MODIFIED Requirements

### Requirement: New game and win handling
The GUI SHALL let the player start a fresh game (a new random seed) at any time, and SHALL, on a win reached by play, display a win indication with the final score (including the timed bonus when timed) and the elapsed time. On a win reached by play the GUI SHALL first play a celebration animation (see gui-animation); the win banner SHALL be shown when the celebration ends — either when the player clicks/taps to dismiss it or after it finishes on its own — so both paths reach the same end-state. The settled celebration cards SHALL remain on screen beneath the banner and SHALL be cleared only when the player starts a new game. The win banner SHALL also show the persisted high score (see gui-persistence), and SHALL indicate when the just-finished game set a new high score. A win reached by auto-solve SHALL NOT play the celebration and SHALL instead be indicated as auto-solved rather than presented as a scored win.

#### Scenario: New game re-deals
- **WHEN** the player starts a new game
- **THEN** a fresh deal appears and the move count and score reset

#### Scenario: Win is shown
- **WHEN** all four foundations are completed by play and the celebration has ended
- **THEN** a win banner appears showing the final score and elapsed time, with the settled cards still visible beneath it

#### Scenario: New game clears the settled cards
- **WHEN** the player starts a new game after a celebration
- **THEN** the settled cards are removed and a fresh deal appears

#### Scenario: Celebration precedes the banner on a played win
- **WHEN** all four foundations are completed by play
- **THEN** the celebration animation plays, and the win banner is shown once it ends (by click/tap or on its own)

#### Scenario: High score is shown on the banner
- **WHEN** the win banner is shown
- **THEN** it displays the persisted high score

#### Scenario: New high score is indicated
- **WHEN** the just-finished played game set a new high score
- **THEN** the win banner indicates that a new high score was achieved

#### Scenario: Auto-solved finish is distinct
- **WHEN** the game is completed by auto-solve
- **THEN** the finish is indicated as auto-solved, not shown as a scored win, and no celebration plays

### Requirement: Settings
The GUI SHALL provide a settings dialog with: the draw mode (one or three) applied
to the next new game (the current game is unchanged); a toggle for the background
solver; a show/hide toggle for the seed; and a deck control with three states —
Auto, Mobile, and Standard — where Auto follows the automatic width-based deck
selection and the other two force that card set. The deck control SHALL take effect
immediately on the current game. Settings SHALL take effect within the session and
SHALL be persisted across launches (see gui-persistence): the current values are
saved when changed and restored on the next launch.

#### Scenario: Draw-mode setting applies to the next game
- **WHEN** the player changes the draw mode in settings and then starts a new game
- **THEN** the new deal uses the chosen draw mode while the prior game was unaffected

#### Scenario: Solver toggle takes effect
- **WHEN** the player disables the background solver in settings
- **THEN** background checks stop

#### Scenario: Seed visibility toggle takes effect
- **WHEN** the player toggles show/hide seed
- **THEN** the status area shows or hides the seed accordingly

#### Scenario: Deck override takes effect immediately
- **WHEN** the player sets the deck control to Mobile or Standard
- **THEN** the current game's cards immediately render from that set, overriding the automatic width-based choice

#### Scenario: Deck Auto follows the viewport
- **WHEN** the player sets the deck control back to Auto
- **THEN** the deck reverts to the automatic width-based selection for the current viewport

#### Scenario: Settings are restored on the next launch
- **WHEN** the player changes settings and later relaunches the game
- **THEN** the settings dialog reflects the previously chosen values
