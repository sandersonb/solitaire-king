# gui-rendering Specification

## Purpose

How the GUI draws the game: the overall board layout (including mobile/portrait
adaptation and the on-screen control bar), card rendering (face-up, face-down,
empty, with sprite, mobile-set, and procedural-fallback paths), text rendering
with a bundled font, and the status/seed/drag feedback display (metrics, seed as a
pronounceable string, dragged-card indication, rejected-move messages).

## Requirements

### Requirement: Board layout
The GUI SHALL render the full board: the four foundations and the stock and waste in an upper area, and the seven tableau columns below, with tableau cards overlapped so face-up cards are readable and the column grows downward. The layout SHALL scale to the window size, and SHALL adapt to narrow / portrait (touch) viewports — reserving space for an on-screen control bar and sizing cards for small-screen readability — so the game is playable on a phone.

#### Scenario: All piles are visible
- **WHEN** a game is rendered
- **THEN** the stock, waste, four foundations, and seven tableau columns are all visible and positioned distinctly

#### Scenario: Tableau overlap
- **WHEN** a tableau column holds several cards
- **THEN** the cards overlap vertically so each is identifiable and the column reads top-to-bottom

#### Scenario: Mobile / portrait layout
- **WHEN** the viewport is narrow or portrait (a typical phone)
- **THEN** the board adapts to fit, an on-screen control bar is shown, and cards are sized to remain readable

### Requirement: Card rendering
Face-up cards SHALL show their rank and suit; face-down cards SHALL show a card back; empty piles SHALL show a placeholder outline. Red suits (hearts, diamonds) SHALL be visually distinct from black suits (clubs, spades). Cards SHALL render from image sprites when available, and SHALL fall back to a procedurally drawn card (rank and suit on a card shape) when sprites are absent, so the game is always playable. On mobile / touch viewports the GUI SHALL prefer a higher-legibility mobile card image set when present, falling back to the desktop card set, then to the procedural card.

#### Scenario: Face-up vs face-down vs empty
- **WHEN** a column has face-down cards beneath a face-up card and another pile is empty
- **THEN** the face-down cards show a back, the face-up card shows its rank and suit, and the empty pile shows a placeholder

#### Scenario: Sprite and fallback paths
- **WHEN** card image assets are present
- **THEN** cards render from those sprites
- **WHEN** the assets are absent
- **THEN** cards render procedurally and the game remains fully playable

#### Scenario: Mobile card art preferred on touch
- **WHEN** the game runs on a mobile / touch viewport and the mobile card set is present
- **THEN** cards render from the mobile set; when it is absent the desktop set (then the procedural card) is used

### Requirement: Status, seed, and drag feedback display
The GUI SHALL display the seed (as a pronounceable seed string), move count, score, and elapsed time, SHALL visually indicate the card or run currently being dragged, and SHALL show a brief message when a move is rejected or an action is not possible.

#### Scenario: Dragged card is indicated
- **WHEN** the player is dragging a card or run
- **THEN** the dragged card(s) are drawn following the pointer, visually distinct from the cards left in place

#### Scenario: Rejected move feedback
- **WHEN** the player attempts an illegal move
- **THEN** a brief message indicates it was not allowed and the board is unchanged

#### Scenario: Seed shown as a readable string
- **WHEN** the status area is rendered
- **THEN** the seed is shown as the pronounceable seed string

### Requirement: Text rendering with a bundled font
The GUI SHALL render its text (splash, status line, on-screen buttons, and the procedural card fallback) using a bundled, legible font so text is crisp at high DPI. If the font asset is missing, the GUI SHALL fall back to a built-in font and remain fully readable.

#### Scenario: Bundled font used for text
- **WHEN** the GUI renders text and the bundled font is present
- **THEN** the text is drawn with that font

#### Scenario: Missing font falls back
- **WHEN** the bundled font asset is absent
- **THEN** the GUI renders text with a built-in font and remains readable

### Requirement: On-screen control bar rendering
The GUI SHALL render an on-screen control bar containing the touch-usable buttons (at least Undo and New), positioned so it does not overlap the playable piles, and SHALL give visual feedback when a button is pressed.

#### Scenario: Control bar is drawn and does not cover piles
- **WHEN** the board is rendered on a layout that shows the control bar
- **THEN** the Undo and New buttons are visible and do not overlap the stock, waste, foundations, or tableau
