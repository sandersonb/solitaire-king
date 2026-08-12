## ADDED Requirements

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
