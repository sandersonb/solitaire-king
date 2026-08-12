## ADDED Requirements

### Requirement: Depth-first brute-force search
The solver SHALL explore a deal by depth-first search: from a position it generates the candidate moves, applies one to a successor position, and recurses, backtracking on return. The search SHALL find a winning line if one exists and is reachable within the configured budget, and SHALL stop as soon as the first winning line is found.

#### Scenario: Finds a win on a solvable position
- **WHEN** the solver runs on a position from which a win is reachable within budget
- **THEN** it reports the position as solvable and returns the winning moveset it found

#### Scenario: Stops at the first win
- **WHEN** a winning line is reached
- **THEN** the search returns immediately without exploring further branches

#### Scenario: Applies only legal moves
- **WHEN** the solver expands a position
- **THEN** every move it applies is one the rules engine reports as legal

### Requirement: Per-path cycle detection
The solver SHALL keep the set of encoded positions on the current search path (the DFS ancestors). Before recursing into a successor, if that successor's encoding already appears on the current path, the solver SHALL discard that branch as a cycle. On backtracking, the position SHALL be removed from the path set.

#### Scenario: A revisited position is pruned
- **WHEN** applying a move would produce a position already present on the current search path
- **THEN** that branch is not explored

#### Scenario: The same position on a different path is allowed
- **WHEN** a position appears on a different branch than the one currently being explored
- **THEN** it may still be explored (cycle detection is per-path, not global)

### Requirement: No-op move classification
The solver SHALL classify a move as a **no-op** and skip it when all of the following hold: (a) it is a tableau-to-tableau move; (b) it reveals no face-down card (the card beneath the moved run is face-up, or the source column becomes empty); (c) it lands on an interchangeable host — either an empty column moved to another empty column, or the moved run's bottom card moves from a card onto another card of the same rank and same color; and (d) the card exposed at the source after the move (if any) has no legal move. Stock draws, recycles, moves to a foundation, moves from a foundation, and any move that reveals a face-down card SHALL never be classified as no-ops.

#### Scenario: King between empty columns is a no-op
- **WHEN** a King (or a King-headed run) is moved from an otherwise-empty column to another empty column
- **THEN** the move is classified as a no-op and skipped

#### Scenario: Lateral shift between equivalent hosts is a no-op
- **WHEN** a red 2 is moved from one black 3 to another black 3, revealing no face-down card, and the exposed black 3 has no legal move
- **THEN** the move is classified as a no-op and skipped

#### Scenario: A revealing move is never a no-op
- **WHEN** a tableau move exposes and flips a face-down card
- **THEN** the move is not a no-op and is explored

#### Scenario: Stock and foundation moves are never no-ops
- **WHEN** the candidate move draws from the stock, recycles, or targets a foundation
- **THEN** it is not classified as a no-op

### Requirement: Equivalence pruning is optional and off by default
The solver SHALL support an **equivalence** pruning rule — collapsing interchangeable destinations for a freshly available card so only one is explored — but it SHALL be disabled by default because its soundness is unproven. When disabled, the search SHALL consider all such destinations.

#### Scenario: Equivalence pruning disabled by default
- **WHEN** the solver runs with default settings
- **THEN** equivalence pruning is not applied and every legal destination for a drawn card is considered

#### Scenario: Equivalence pruning can be enabled
- **WHEN** the solver is configured to enable equivalence pruning
- **THEN** interchangeable destinations for a freshly available card are collapsed to one

### Requirement: Differential pruning validation
The solver SHALL provide a validation mode that runs a search with and without a given optional pruning rule over one or more seeds and reports whether the solvable/unsolvable verdict is unchanged, so a pruning hypothesis can be empirically checked.

#### Scenario: Validation flags a difference
- **WHEN** an optional pruning rule changes whether a win is found for some seed within the same budget
- **THEN** the validation reports a discrepancy for that seed

#### Scenario: Validation confirms agreement
- **WHEN** an optional pruning rule does not change the solvable verdict for any tested seed
- **THEN** the validation reports agreement

### Requirement: Search budget
The solver SHALL accept a budget — a maximum number of expanded nodes and a maximum elapsed time — and SHALL stop when either limit is reached (and immediately once a win is found). If no win was found when the budget is exhausted, the result SHALL be reported as inconclusive rather than as proven unwinnable.

#### Scenario: Stops at the node limit
- **WHEN** the number of expanded nodes reaches the configured maximum before a win is found
- **THEN** the search stops and reports that the budget was exhausted

#### Scenario: Budget exhaustion is inconclusive
- **WHEN** the budget is exhausted without finding a win
- **THEN** the result indicates the outcome is inconclusive (not proven unwinnable)

### Requirement: First winning line and result
The solver SHALL stop at the first winning move-sequence it discovers and retain it. The result SHALL report: solvable (whether a win was found within budget), the winning moveset when solvable, and search statistics — nodes expanded, elapsed time, and peak logical memory.

#### Scenario: Returns the first winning line
- **WHEN** the solver finds a win within budget
- **THEN** it reports the deal as solvable and returns the winning moveset it found

#### Scenario: Winning moveset is replayable
- **WHEN** the returned winning moveset is applied in order to the original deal
- **THEN** every move is legal and the final state is won

### Requirement: Logical memory accounting
The solver SHALL measure memory logically and portably (no OS calls): it SHALL track the peak bytes held in its own structures — the per-path position set, the DFS working state, and the retained winning moveset (if any) — reported as a peak-bytes figure (optionally with the peak number of positions held).

#### Scenario: Reports a peak memory figure
- **WHEN** a search completes
- **THEN** the result includes a peak logical memory value derived from the solver's own structures, not from OS process memory
