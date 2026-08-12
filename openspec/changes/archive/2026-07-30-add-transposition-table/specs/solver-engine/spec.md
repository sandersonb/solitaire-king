## ADDED Requirements

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
