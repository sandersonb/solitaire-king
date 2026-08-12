## ADDED Requirements

### Requirement: Zobrist position hashing
The solver SHALL provide a 128-bit Zobrist hash of a position, computed by combining (XOR) a deterministic random value per position feature: each tableau/stock/waste card by its (card, pile, depth, face-up) placement, and each foundation by its (suit, top rank). The hash SHALL use the same canonicalization as the byte encoding: it depends only on the playable position (not score or elapsed time), treats foundation slots as interchangeable (keyed by suit), and includes the recycles-remaining only when the redeal limit is bounded. The random feature table SHALL be fixed (deterministic) so a given position always hashes the same.

#### Scenario: Same position, same hash
- **WHEN** a position is hashed twice
- **THEN** the two 128-bit hashes are identical

#### Scenario: Hash identity matches the byte encoding
- **WHEN** two positions are compared
- **THEN** they have equal Zobrist hashes exactly when they have equal byte encodings (barring an astronomically unlikely 128-bit collision)

#### Scenario: Score and time excluded
- **WHEN** two positions have identical piles but different scores or elapsed times
- **THEN** their Zobrist hashes are equal

#### Scenario: Foundation slots interchangeable
- **WHEN** two positions hold the same foundation contents in different foundation slots
- **THEN** their Zobrist hashes are equal
