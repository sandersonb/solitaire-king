## MODIFIED Requirements

### Requirement: Drag to move a card
The GUI SHALL move cards by direct manipulation: pressing on a face-up tableau
card picks up that card together with the face-up run above it; pressing on the
waste's top card or a foundation's top card picks up that card. While the pointer
is held, the picked-up card(s) SHALL follow the pointer. Releasing near a legal
destination pile SHALL apply the corresponding move; the card SHALL NOT need to be
dropped precisely on the pile — release within the pile's drop zone SHALL count.
Only legal moves SHALL be applied; releasing where no legal move exists SHALL
leave the board unchanged and return the card(s) to their origin. Input SHALL work
with both a mouse and touch.

#### Scenario: Picking up a tableau run
- **WHEN** the player presses a face-up card that has more face-up cards on top of it
- **THEN** that card and the cards above it are picked up together and follow the pointer

#### Scenario: Drop near a legal destination applies the move
- **WHEN** the player releases a picked-up card within the drop zone of a pile that forms a legal move
- **THEN** the move is applied even though the release point was not exactly on the pile

#### Scenario: Illegal release is rejected
- **WHEN** the player releases a picked-up card where no legal move exists
- **THEN** no move is applied, a brief message is shown, and the card returns to its origin

#### Scenario: Touch drag works
- **WHEN** the player drags a card with a touch gesture on a touch device
- **THEN** the card follows the touch and the move resolves the same as with a mouse

### Requirement: Auto-move
Double-clicking or double-tapping a face-up card, or pressing the auto-move key (Enter) with a card picked up (or the waste's top when nothing is picked up), SHALL move that card to its best legal destination — preferring a foundation, then a tableau — matching the CLI's auto-assign. If no legal destination exists, no move SHALL be made and a message SHALL be shown.

#### Scenario: Double-click auto-moves
- **WHEN** the player double-clicks or double-taps a face-up card that has a legal destination
- **THEN** it is moved to its best legal destination (foundation preferred)

#### Scenario: Auto-move with no destination
- **WHEN** the player triggers auto-move on a card with no legal destination
- **THEN** no move is made and a brief message is shown

## ADDED Requirements

### Requirement: On-screen controls
The GUI SHALL provide on-screen buttons for the core session actions that need no
keyboard — at minimum **Undo** and **New game** — so the game is fully playable by
touch alone. Redo SHALL NOT require an on-screen button. Activating a button
(click or tap) SHALL perform its action; the buttons SHALL be present on touch /
mobile layouts and MAY be shown on desktop.

#### Scenario: Undo button works by touch
- **WHEN** the player taps the on-screen Undo button after a move
- **THEN** the last move is undone, with no keyboard required

#### Scenario: New game button works by touch
- **WHEN** the player taps the on-screen New button
- **THEN** a fresh game is dealt

## REMOVED Requirements

### Requirement: Click to select a source
**Reason**: Replaced by direct-manipulation drag-and-drop; selecting a source and
then a target in two separate clicks (with a persistent selection highlight) is
superseded by pressing to pick up and releasing to drop.
**Migration**: Players pick up a card by pressing on it and drop it by releasing
over the destination; there is no longer a separate select step or selection
highlight. Auto-move (double-click/tap, Enter) is unchanged.

### Requirement: Click to move to a destination
**Reason**: Merged into the drag-to-move behavior — the destination is chosen by
where the card is released, not by a second click.
**Migration**: Release a picked-up card over (or near) the destination pile;
legality and rejection feedback behave as before.
