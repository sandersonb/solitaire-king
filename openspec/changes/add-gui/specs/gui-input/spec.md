## ADDED Requirements

### Requirement: Click to select a source
Clicking a face-up tableau card SHALL select that card together with the face-up run above it; clicking the waste's top card SHALL select it; clicking a foundation's top card SHALL select it. A selection SHALL be visually indicated, and clicking empty space or the same source again SHALL clear it.

#### Scenario: Selecting a tableau run
- **WHEN** the player clicks a face-up card that has more face-up cards on top of it
- **THEN** that card and the cards above it are selected as a run

#### Scenario: Deselecting
- **WHEN** a selection is active and the player clicks empty space
- **THEN** the selection is cleared

### Requirement: Click to move to a destination
With a source selected, clicking a destination pile (a tableau column or a foundation) SHALL attempt the corresponding move. Only legal moves SHALL be applied; an illegal target SHALL leave the board unchanged and surface brief feedback.

#### Scenario: Legal move applies
- **WHEN** a source is selected and the player clicks a destination that forms a legal move
- **THEN** the move is applied and the selection clears

#### Scenario: Illegal move is rejected
- **WHEN** a source is selected and the player clicks a destination that is not a legal move
- **THEN** no move is applied and a brief message is shown

### Requirement: Deal from the stock
Clicking the stock SHALL deal from it to the waste per the draw mode; when the stock is empty, clicking it SHALL recycle the waste back into the stock (subject to the redeal limit).

#### Scenario: Click deals
- **WHEN** the stock is non-empty and the player clicks it
- **THEN** cards are dealt to the waste according to the draw mode

#### Scenario: Click recycles when empty
- **WHEN** the stock is empty and the player clicks it
- **THEN** the waste is recycled into the stock (if the redeal limit permits)

### Requirement: Auto-move
Double-clicking a face-up card, or pressing the auto-move key (Enter) with a card selected (or the waste's top when nothing is selected), SHALL move that card to its best legal destination — preferring a foundation, then a tableau — matching the CLI's auto-assign. If no legal destination exists, no move SHALL be made and a message SHALL be shown.

#### Scenario: Double-click auto-moves
- **WHEN** the player double-clicks a face-up card that has a legal destination
- **THEN** it is moved to its best legal destination (foundation preferred)

#### Scenario: Auto-move with no destination
- **WHEN** the player triggers auto-move on a card with no legal destination
- **THEN** no move is made and a brief message is shown

### Requirement: Keyboard commands
The GUI SHALL provide keyboard commands for the core session actions: undo, redo, new game, and auto-move, so a player can drive the game without only the mouse.

#### Scenario: Undo via keyboard
- **WHEN** the player presses the undo key after a move
- **THEN** the last move is undone
