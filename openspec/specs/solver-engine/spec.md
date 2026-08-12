# solver-engine Specification

## Purpose

Defines the depth-first brute-force search engine that decides whether a Klondike deal is solvable within a budget: candidate move expansion, per-path cycle detection, no-op and optional equivalence pruning (with differential validation), search budgeting, first-winning-line capture, and logical memory accounting.

## Requirements

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

### Requirement: Global transposition table
The solver SHALL maintain a global transposition table recording positions that have been fully explored and found to contain no reachable win (a "closed" set). A position SHALL be added to the table only after its entire subtree has been explored without a win. Before expanding a position, if it is present in the table, the solver SHALL skip it. The table SHALL be keyed by the position encoding. This behavior SHALL be on by default and disableable (disabling reproduces the per-path baseline).

#### Scenario: A closed position is skipped on later encounters
- **WHEN** a position was fully explored on one branch and found winless, and the same position is reached again on another branch
- **THEN** the solver skips it instead of re-exploring its subtree

#### Scenario: Open positions are not yet closed
- **WHEN** a position is still being explored (its subtree is not finished)
- **THEN** it is not yet in the closed table

#### Scenario: Disabling the table reproduces the baseline
- **WHEN** the transposition table is disabled
- **THEN** the search behaves as the per-path solver (only the on-path cycle guard and existing pruning remain)

### Requirement: On-path cycle guard is independent of the table
The solver SHALL keep a separate set of the positions on the current search path (its ancestors) that is never evicted, and SHALL use it to prevent revisiting a position on the current path. Termination SHALL be guaranteed by this on-path set, independent of the transposition table's contents or eviction.

#### Scenario: Cycle prevented even if the table evicted the position
- **WHEN** a move would return to a position on the current path, whether or not that position is in the transposition table
- **THEN** the solver does not recurse into it (no infinite loop)

### Requirement: Bounded table with sound eviction
The transposition table SHALL have a configurable maximum size and SHALL evict entries when full. Because the table holds only proven-winless positions, eviction SHALL only cause a position to be re-explored later — it SHALL never cause a win to be missed nor a position to be wrongly skipped.

#### Scenario: Insertion into a full table evicts, not fails
- **WHEN** a winless position is recorded and the table is at capacity
- **THEN** an existing entry is evicted to make room and the search continues correctly

#### Scenario: Eviction preserves soundness
- **WHEN** the table is run with a small capacity versus a large capacity over the same deal and budget
- **THEN** the solvable/unwinnable verdict is the same (a small table is only slower, never wrong)

### Requirement: Proven-unwinnable verdict
When the search explores the entire reachable state space within the budget and finds no win, the solver SHALL report the deal as **proven unwinnable**. When instead a node or time budget is reached before the space is exhausted, the solver SHALL report **inconclusive**. A solved deal SHALL report **solvable**. These three outcomes SHALL be distinguishable in the result.

#### Scenario: Exhausted search with no win is proven unwinnable
- **WHEN** the search completes (no more unexplored, non-cyclic positions) without a win and no budget limit was hit
- **THEN** the result's verdict is Unwinnable (proven)

#### Scenario: Budget hit before exhaustion is inconclusive
- **WHEN** a node or time limit is reached before the reachable space is exhausted and no win was found
- **THEN** the result's verdict is Inconclusive, not Unwinnable

#### Scenario: A found win is solvable
- **WHEN** a winning line is found
- **THEN** the result's verdict is Solvable and a winning moveset is returned

### Requirement: Table statistics
The solver SHALL report transposition-table statistics: the number of entries retained (peak), the number of probe hits (positions skipped because they were in the table), and the number of evictions.

#### Scenario: Stats reflect table activity
- **WHEN** a search completes with the table enabled
- **THEN** the result includes the peak entry count, the hit count, and the eviction count

### Requirement: In-place search via make/unmake
The search SHALL explore successors by applying a move to a single mutable game state and undoing it on backtrack (make/unmake), rather than cloning a new state per candidate move. The set of nodes explored and the verdict reached SHALL be identical to the cloning search (make/unmake is a pure optimization).

#### Scenario: No per-child cloning
- **WHEN** the search expands a node's candidate moves
- **THEN** it applies and later undoes each move on one shared state rather than allocating a cloned child per move

#### Scenario: Make/unmake does not change results
- **WHEN** a deal is solved with the make/unmake search
- **THEN** its verdict matches what the cloning search would produce for the same options and budget

### Requirement: Selectable position key
The solver SHALL support a selectable key strategy for its transposition table and on-path set: a 128-bit Zobrist hash (the default) or the exact byte encoding. Both SHALL yield the same verdict on a given position (the byte encoding is a collision-free reference); Zobrist SHALL be used by default for its speed and compactness.

#### Scenario: Zobrist is the default key
- **WHEN** the solver runs with default options
- **THEN** positions are keyed by the 128-bit Zobrist hash

#### Scenario: Exact-byte keying is selectable
- **WHEN** the solver is configured to use exact byte keys
- **THEN** positions are keyed by the byte encoding (no hash collisions possible)

### Requirement: Key-strategy differential validation
The differential validator SHALL be able to compare the Zobrist and exact-byte key strategies, confirming they reach the same verdict on positions that search to completion, so any hash-collision effect would be detected.

#### Scenario: Zobrist and exact keys agree
- **WHEN** a completing position is solved with Zobrist keys and with exact-byte keys under the same budget
- **THEN** the two verdicts are identical
