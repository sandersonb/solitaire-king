## ADDED Requirements

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
