## ADDED Requirements

### Requirement: Safe foundation auto-moves
The solver SHALL treat a foundation move as **safe** — playable without ever being needed back in the tableau — when the card's rank is at most 2, or both foundations of the **opposite color** are at rank ≥ (card rank − 1). When a safe foundation move is available at a node, the solver SHALL play it as the node's only move (forced, no branching). Safe auto-moves SHALL never change whether a win is found (they are sound). This behavior SHALL be on by default and disableable.

#### Scenario: Aces and twos are always safe
- **WHEN** an Ace or a 2 can be played to a foundation
- **THEN** it is treated as a safe auto-move and forced

#### Scenario: Higher card safe once opposite colors have caught up
- **WHEN** a card of rank r (r ≥ 3) can go to its foundation and both opposite-color foundations are at rank ≥ r − 1
- **THEN** it is treated as a safe auto-move and forced, even if a same-color foundation lags

#### Scenario: Unsafe foundation move is not forced
- **WHEN** a card could go to its foundation but an opposite-color foundation is below rank − 1
- **THEN** it is NOT auto-forced and remains an ordinary (branchable) candidate

#### Scenario: Forcing safe auto-moves preserves the verdict
- **WHEN** the solver runs with safe auto-moves on versus off, over the same deal and budget
- **THEN** the solvable verdict is unchanged

### Requirement: Heuristic move ordering
When not forcing a safe auto-move, the solver SHALL order the candidate moves by a priority heuristic so promising lines are explored first, without ever discarding a legal move (completeness preserved). The ordering SHALL rank: moves that reveal a face-down tableau card first (tie-broken by the configured digging direction), then other productive builds and waste plays, then stock draws (with a penalty when the previous move was also a draw), and finally foundation→tableau moves. Move ordering SHALL be on by default and disableable to reproduce the baseline.

#### Scenario: Revealing moves are tried before drawing
- **WHEN** both a face-down-revealing tableau move and a stock draw are available
- **THEN** the revealing move is ordered ahead of the draw

#### Scenario: Consecutive draws are deprioritized
- **WHEN** the previous move was a stock draw and other productive moves exist
- **THEN** another draw is ordered after those productive moves

#### Scenario: Foundation-to-tableau is last resort
- **WHEN** a foundation→tableau move and any other legal move are both available
- **THEN** the foundation→tableau move is ordered last

#### Scenario: Ordering does not change the verdict
- **WHEN** the solver runs with move ordering on versus off, over the same deal and budget
- **THEN** the solvable verdict is unchanged (ordering only reorders, never removes, moves)

### Requirement: Digging direction is configurable
When two candidate moves each reveal a face-down card in different source columns, the solver SHALL break the tie by a configurable preference: by default it SHALL prefer revealing the card in the **larger** (taller) source column; the alternative SHALL prefer the smaller column.

#### Scenario: Default prefers the larger stack
- **WHEN** two revealing moves are available in columns of different heights and the default digging direction is in effect
- **THEN** the move revealing a card in the taller column is ordered first

#### Scenario: Digging direction can be inverted
- **WHEN** the digging direction is set to prefer smaller columns
- **THEN** the move revealing a card in the shorter column is ordered first

### Requirement: Empty-column symmetry pruning
Because empty tableau columns are interchangeable, when more than one column is empty the solver SHALL consider moving a King (or King-headed run) into only one of them, rather than generating an equivalent move for each empty column. This SHALL be sound (it never removes the only path to a win) and on by default.

#### Scenario: Only one empty target for a King
- **WHEN** a King can be moved to an empty column and two or more columns are empty
- **THEN** the solver explores that move to a single empty column, not once per empty column

#### Scenario: Symmetry pruning preserves the verdict
- **WHEN** the solver runs with empty-column symmetry on versus off, over the same deal and budget
- **THEN** the solvable verdict is unchanged
