# gui-input Specification

## Purpose

Mouse, touch, and keyboard interaction for the GUI: drag-to-move a card or run,
dealing/recycling from the stock, auto-move (double-click/tap or key), on-screen
controls for touch, and keyboard commands for the core session actions.

## Requirements

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

### Requirement: Deal from the stock
Clicking the stock SHALL deal from it to the waste per the draw mode; when the stock is empty, clicking it SHALL recycle the waste back into the stock (subject to the redeal limit).

#### Scenario: Click deals
- **WHEN** the stock is non-empty and the player clicks it
- **THEN** cards are dealt to the waste according to the draw mode

#### Scenario: Click recycles when empty
- **WHEN** the stock is empty and the player clicks it
- **THEN** the waste is recycled into the stock (if the redeal limit permits)

### Requirement: Auto-move
Double-clicking or double-tapping a face-up card, or pressing the auto-move key (Enter) with a card picked up (or the waste's top when nothing is picked up), SHALL move that card to its best legal destination — preferring a foundation, then a tableau — matching the CLI's auto-assign. If no legal destination exists, no move SHALL be made and a message SHALL be shown.

#### Scenario: Double-click auto-moves
- **WHEN** the player double-clicks or double-taps a face-up card that has a legal destination
- **THEN** it is moved to its best legal destination (foundation preferred)

#### Scenario: Auto-move with no destination
- **WHEN** the player triggers auto-move on a card with no legal destination
- **THEN** no move is made and a brief message is shown

### Requirement: Keyboard commands
The GUI SHALL provide keyboard commands for the core session actions: undo, redo, new game, and auto-move, so a player can drive the game without only the mouse.

#### Scenario: Undo via keyboard
- **WHEN** the player presses the undo key after a move
- **THEN** the last move is undone

### Requirement: On-screen controls
The GUI SHALL provide on-screen buttons for the core session actions so the game is
fully playable by touch alone: a combined **Undo/Redo** button where a tap undoes the
last move and a press-and-hold redoes one move; a **New game** button; and a
**Settings** button. The solvability indicator SHALL also act as a button. Activating
any control (click or tap) SHALL perform its action; the controls SHALL be present on
touch / mobile layouts and MAY be shown on desktop.

#### Scenario: Undo button works by touch
- **WHEN** the player taps the on-screen Undo button after a move
- **THEN** the last move is undone, with no keyboard required

#### Scenario: Hold redoes
- **WHEN** the player presses and holds the Undo button after undoing a move
- **THEN** one undone move is redone

#### Scenario: New game button works by touch
- **WHEN** the player taps the on-screen New button
- **THEN** a fresh game is dealt

#### Scenario: Settings button opens settings
- **WHEN** the player taps the Settings button
- **THEN** the settings dialog opens

### Requirement: Interactive solver actions
Activating the solvability indicator SHALL open the state-dependent solver overlay
(except while a check runs). When a solution is known, the overlay's Auto-solve
action and the **Shift+A** shortcut SHALL start auto-solving the current deal; Shift+A
SHALL do nothing when no solution is known. Overlays SHALL be dismissible (a close
control or clicking outside), and while an overlay or dialog is open it SHALL take
input priority over the board.

#### Scenario: Indicator opens the overlay
- **WHEN** the player activates the indicator while no check is running
- **THEN** the solver overlay for the current status opens

#### Scenario: Auto-solve from the overlay
- **WHEN** the player activates Auto-solve in the solvable overlay
- **THEN** auto-solving begins

#### Scenario: Shift+A auto-solves when a solution is known
- **WHEN** the player presses Shift+A and a solution is known for the current position
- **THEN** auto-solving begins

#### Scenario: Overlay takes input priority
- **WHEN** an overlay or dialog is open and the player clicks
- **THEN** the click is handled by the overlay, not the board
