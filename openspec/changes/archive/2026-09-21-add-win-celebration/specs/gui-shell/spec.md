## MODIFIED Requirements

### Requirement: New game and win handling
The GUI SHALL let the player start a fresh game (a new random seed) at any time, and SHALL, on a win reached by play, display a win indication with the final score (including the timed bonus when timed) and the elapsed time. On a win reached by play the GUI SHALL first play a celebration animation (see gui-animation); the win banner SHALL be shown when the celebration ends — either when the player clicks/taps to dismiss it or after it finishes on its own — so both paths reach the same end-state. The settled celebration cards SHALL remain on screen beneath the banner and SHALL be cleared only when the player starts a new game. A win reached by auto-solve SHALL NOT play the celebration and SHALL instead be indicated as auto-solved rather than presented as a scored win.

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

#### Scenario: Auto-solved finish is distinct
- **WHEN** the game is completed by auto-solve
- **THEN** the finish is indicated as auto-solved, not shown as a scored win, and no celebration plays
