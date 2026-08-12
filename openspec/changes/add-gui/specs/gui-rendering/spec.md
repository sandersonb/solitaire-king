## ADDED Requirements

### Requirement: Board layout
The GUI SHALL render the full board: the four foundations and the stock and waste in an upper area, and the seven tableau columns below, with tableau cards overlapped so face-up cards are readable and the column grows downward. The layout SHALL scale to the window size.

#### Scenario: All piles are visible
- **WHEN** a game is rendered
- **THEN** the stock, waste, four foundations, and seven tableau columns are all visible and positioned distinctly

#### Scenario: Tableau overlap
- **WHEN** a tableau column holds several cards
- **THEN** the cards overlap vertically so each is identifiable and the column reads top-to-bottom

### Requirement: Card rendering
Face-up cards SHALL show their rank and suit; face-down cards SHALL show a card back; empty piles SHALL show a placeholder outline. Red suits (hearts, diamonds) SHALL be visually distinct from black suits (clubs, spades). Cards SHALL render from image sprites when available, and SHALL fall back to a procedurally drawn card (rank and suit on a card shape) when sprites are absent, so the game is always playable.

#### Scenario: Face-up vs face-down vs empty
- **WHEN** a column has face-down cards beneath a face-up card and another pile is empty
- **THEN** the face-down cards show a back, the face-up card shows its rank and suit, and the empty pile shows a placeholder

#### Scenario: Sprite and fallback paths
- **WHEN** card image assets are present
- **THEN** cards render from those sprites
- **WHEN** the assets are absent
- **THEN** cards render procedurally and the game remains fully playable

### Requirement: Status and feedback display
The GUI SHALL display the seed, move count, score, and elapsed time, SHALL highlight the currently selected card or run, and SHALL show a brief message when a move is rejected or an action is not possible.

#### Scenario: Selection is highlighted
- **WHEN** the player has selected a card or run and not yet chosen a destination
- **THEN** the selected card(s) are visually highlighted

#### Scenario: Rejected move feedback
- **WHEN** the player attempts an illegal move
- **THEN** a brief message indicates it was not allowed and the board is unchanged
