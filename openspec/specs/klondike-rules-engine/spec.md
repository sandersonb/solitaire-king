# klondike-rules-engine Specification

## Purpose

Defines the classic (standard) Klondike ruleset engine: the move vocabulary, legal-move enumeration, tableau and foundation placement rules, move application (including auto-flip and stock recycle), redeal-limit enforcement, and win detection.

## Requirements

### Requirement: Move vocabulary
The system SHALL represent every player action as a `Move` value drawn from the classic Klondike action set: draw from stock to waste, recycle the waste back into the stock, move the waste's top card to a foundation, move the waste's top card to a tableau column, move a tableau card (or an ordered face-up run) to another tableau column, move a tableau card to a foundation, and move a foundation's top card back to a tableau column.

#### Scenario: Every action is expressible
- **WHEN** any legal classic-Klondike action is requested
- **THEN** it corresponds to exactly one `Move` variant with the operands needed to apply it

### Requirement: Legal move enumeration
The system SHALL enumerate all currently legal moves for a given `GameState` under the classic ruleset, so callers (human UI or solver) can choose among valid options. The enumeration SHALL include a stock draw when the stock is non-empty, a stock recycle only when the stock is empty and the redeal limit has not been reached, and all foundation/tableau placements permitted by the placement rules below.

#### Scenario: Draw available with non-empty stock
- **WHEN** the stock contains at least one card
- **THEN** the legal-move list includes a stock-draw move

#### Scenario: No recycle at redeal limit
- **WHEN** the stock is empty and the configured redeal limit has already been reached
- **THEN** the legal-move list does not include a recycle move

### Requirement: Tableau placement rules
The system SHALL permit placing a card onto a tableau column only when the destination is empty and the incoming card is a King, or the destination's top face-up card is one rank higher and of the opposite color to the incoming card. When moving a run of face-up cards between tableau columns, the run SHALL itself be a valid descending alternating-color sequence and the whole run SHALL be moved together.

#### Scenario: Valid alternating-color descending placement
- **WHEN** a red Six is moved onto a tableau column whose top face-up card is a black Seven
- **THEN** the move is legal and the red Six becomes the column's new top card

#### Scenario: Only a King fills an empty column
- **WHEN** a tableau column is empty
- **THEN** only a King (or a run headed by a King) may be placed there, and any non-King placement is rejected

### Requirement: Foundation placement rules
The system SHALL permit placing a card onto a foundation only when the card is an Ace onto an empty foundation, or the card is the same suit as and exactly one rank higher than the foundation's current top card.

#### Scenario: Ace starts a foundation
- **WHEN** an Ace is moved onto an empty foundation
- **THEN** the move is legal and that foundation now builds that suit

#### Scenario: Building a foundation in sequence
- **WHEN** the top of a foundation is the Four of Hearts and the Five of Hearts is moved onto it
- **THEN** the move is legal
- **AND** moving any card that is not the Five of Hearts onto that foundation is rejected

### Requirement: Applying a move
The system SHALL apply a legal `Move` to a `GameState`, transitioning it to the resulting state, and SHALL reject an illegal move without mutating the state. Applying a move SHALL update the affected piles and MAY trigger the auto-flip rule below.

#### Scenario: Illegal move is rejected without side effects
- **WHEN** an illegal move is applied to a state
- **THEN** the operation reports failure
- **AND** the game state is unchanged

### Requirement: Auto-flip exposed tableau card
When a move removes the top face-up card(s) from a tableau column and leaves a face-down card newly exposed at the top, the system SHALL turn that card face-up automatically as part of applying the move.

#### Scenario: Exposing a face-down card flips it
- **WHEN** the last face-up card of a tableau column is moved away and a face-down card remains on top
- **THEN** that newly exposed card is turned face-up

### Requirement: Stock draw and recycle
The system SHALL move cards from the stock to the waste per the configured draw mode. When the stock is empty, a recycle SHALL return the waste to the stock (restoring draw order) and SHALL increment the recycle count, but only if the configured `redeal_limit` permits it; a recycle beyond the limit SHALL be rejected.

#### Scenario: Draw-three moves three cards
- **WHEN** a draw is applied in draw-three mode with at least three cards in the stock
- **THEN** three cards move to the top of the waste in order

#### Scenario: Recycle within unlimited limit
- **WHEN** the stock is empty, the waste is non-empty, and the redeal limit is unlimited
- **THEN** a recycle is legal, returns all waste cards to the stock, and increments the recycle count

#### Scenario: Recycle blocked at limit
- **WHEN** the stock is empty and the number of recycles already performed equals the configured redeal limit
- **THEN** a recycle move is rejected and the state is unchanged

### Requirement: Win detection
The system SHALL report a game as won when all four foundations are complete — each holding 13 cards from Ace through King of a single suit (52 cards total on the foundations).

#### Scenario: All foundations complete
- **WHEN** each of the four foundations contains its full Ace-through-King sequence
- **THEN** the game state reports the game as won

#### Scenario: Incomplete foundations are not a win
- **WHEN** at least one card is not yet on a foundation
- **THEN** the game state does not report a win

### Requirement: Reversible move application
The rules engine SHALL provide an undoable form of move application: applying a legal move SHALL return an undo token, and a corresponding undo operation SHALL, given that token and the move, restore the game state to exactly what it was before the move — including all piles, any auto-flipped tableau card, the score, and the recycle count. The undo token SHALL be a small value type requiring no heap allocation. Illegal moves SHALL still be rejected and leave the state unchanged (no token produced).

#### Scenario: Undo restores the exact prior state
- **WHEN** a legal move is applied with the undoable API and then undone with its token
- **THEN** the resulting state is identical to the state before the move (piles, face-up flags, score, and recycle count all restored)

#### Scenario: Auto-flip is reversed
- **WHEN** a move that auto-flipped a newly exposed tableau card is undone
- **THEN** that card is returned to face-down and the moved card(s) are back in their original pile

#### Scenario: Recycle is reversible
- **WHEN** a stock recycle is applied undoably and then undone
- **THEN** the stock, waste, recycle count, and score are exactly as they were before the recycle

#### Scenario: Illegal move produces no change
- **WHEN** an illegal move is attempted with the undoable API
- **THEN** it reports failure and the state is unchanged

### Requirement: Undoable application matches cloning application
Applying a move via the undoable API SHALL produce the same resulting state as applying it to a clone via the existing API; the two paths SHALL be observationally equivalent for any legal move.

#### Scenario: Make equals clone-and-apply
- **WHEN** the same legal move is applied to a state via the undoable API and to a clone of that state via `apply_move`
- **THEN** the two resulting states are identical
