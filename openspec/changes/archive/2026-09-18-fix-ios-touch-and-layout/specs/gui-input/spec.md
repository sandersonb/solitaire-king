## MODIFIED Requirements

### Requirement: Drag to move a card
The GUI SHALL move cards by direct manipulation: pressing on a face-up tableau
card picks up that card together with the face-up run above it; pressing on the
waste's top card or a foundation's top card picks up that card. While the pointer
is held, the picked-up card(s) SHALL follow the pointer. Releasing near a legal
destination pile SHALL apply the corresponding move; the card SHALL NOT need to be
dropped precisely on the pile — release within the pile's drop zone SHALL count.
When the picked-up card is drawn offset from the pointer (as on touch, where it is
lifted above the finger so the finger does not occlude it), the drop SHALL be
resolved from the card's drawn position rather than the raw pointer position, so
landing the visible card on a pile registers the same forgiving drop as on
desktop. Only legal moves SHALL be applied; releasing where no legal move exists
SHALL leave the board unchanged and return the card(s) to their origin. Input
SHALL work with both a mouse and touch.

#### Scenario: Picking up a tableau run
- **WHEN** the player presses a face-up card that has more face-up cards on top of it
- **THEN** that card and the cards above it are picked up together and follow the pointer

#### Scenario: Drop near a legal destination applies the move
- **WHEN** the player releases a picked-up card within the drop zone of a pile that forms a legal move
- **THEN** the move is applied even though the release point was not exactly on the pile

#### Scenario: Touch drop matches the visible card
- **WHEN** the player drags a card on touch, where it is drawn lifted above the finger, and releases with the visible card overlapping a legal destination pile
- **THEN** the move is applied based on where the card is drawn, not where the finger is, so the drop feels as forgiving as on desktop

#### Scenario: Illegal release is rejected
- **WHEN** the player releases a picked-up card where no legal move exists
- **THEN** no move is applied, a brief message is shown, and the card returns to its origin

#### Scenario: Touch drag works
- **WHEN** the player drags a card with a touch gesture on a touch device
- **THEN** the card follows the touch and the move resolves the same as with a mouse
