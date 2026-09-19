## MODIFIED Requirements

### Requirement: Settings
The GUI SHALL provide a settings dialog with: the draw mode (one or three) applied
to the next new game (the current game is unchanged); a toggle for the background
solver; a show/hide toggle for the seed; and a deck control with three states —
Auto, Detailed, and Standard — where Auto follows the automatic width-based deck
selection and the other two force that card set. The deck control SHALL take effect
immediately on the current game. Settings SHALL take effect within the session;
persistence across launches is not required.

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
- **WHEN** the player sets the deck control to Detailed or Standard
- **THEN** the current game's cards immediately render from that set, overriding the automatic width-based choice, for the rest of the session

#### Scenario: Deck Auto follows the viewport
- **WHEN** the player sets the deck control back to Auto
- **THEN** the deck reverts to the automatic width-based selection for the current viewport
