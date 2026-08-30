# gui-animation Specification

## Purpose

Defines the GUI's card-motion behavior: cards move between screen positions over a
short interval rather than teleporting, so drag-drop snapping reads smoothly and
the same mechanism can later play back automated moves (e.g. a solver's moves).

## Requirements

### Requirement: Animated card motion
When a move changes where a card or run rests on screen, the GUI SHALL animate the
card(s) from their release/origin position to the destination pile's resting
position over a short interval, rather than snapping instantly. While an animation
is in flight the underlying game state SHALL already reflect the applied move, so
input and scoring are never blocked waiting on animation.

#### Scenario: Move animates into place
- **WHEN** a legal move is applied by drag-release
- **THEN** the moved card(s) visibly travel from where they were released to the destination pile's resting position

#### Scenario: Animation does not block play
- **WHEN** a card is mid-animation
- **THEN** the game state already reflects the move and the player can begin the next action

### Requirement: Rejected-move return animation
When the player releases a dragged card where no legal move exists, the GUI SHALL
animate the card(s) back to their origin position and leave the board unchanged.

#### Scenario: Illegal release returns to origin
- **WHEN** a dragged card is released away from any legal destination
- **THEN** it animates back to the pile it came from and no move is applied

### Requirement: Drivable by automated playback
The animation subsystem SHALL be usable to play a queued sequence of moves with
the same motion, independent of pointer input, so a future automated-play feature
(such as animating a solver's solution) can enqueue moves and have them animate in
order.

#### Scenario: Queued moves animate in order
- **WHEN** a sequence of moves is enqueued for automated playback
- **THEN** each move is applied and its card motion animates in the given order
