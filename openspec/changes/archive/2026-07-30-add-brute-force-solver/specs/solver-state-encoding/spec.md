## ADDED Requirements

### Requirement: Compact position encoding
The solver SHALL encode a `GameState` position into a compact, self-contained byte sequence that captures everything relevant to legal play: each tableau column's cards in order with their face-up/face-down flags, the stock's cards in order, the waste's cards in order, and the foundations. The encoding SHALL be deterministic — the same position always produces the same bytes.

#### Scenario: Deterministic encoding
- **WHEN** the same position is encoded twice
- **THEN** the two byte sequences are identical

#### Scenario: Distinct positions differ
- **WHEN** two positions differ in any pile's contents or in any card's face-up state
- **THEN** their encodings differ

### Requirement: Positional-only encoding
The encoding SHALL depend only on the playable position, and SHALL NOT include incidental fields that do not affect legal play (seed, score, elapsed time, move count). Two states that are identical as positions SHALL encode identically even if such incidental fields differ.

#### Scenario: Score and time are excluded
- **WHEN** two positions have identical piles but different scores or elapsed times
- **THEN** their encodings are identical

### Requirement: Foundation canonicalization
Because a foundation is fully determined by its suit and its top rank, and the four foundation slots are interchangeable, the encoding SHALL represent foundations canonically by suit (for example, the highest rank present per suit in a fixed suit order). Two positions that differ only in which foundation slot holds which suit SHALL encode identically.

#### Scenario: Foundation slot order does not matter
- **WHEN** two positions hold the same foundation contents but assigned to different foundation slots
- **THEN** their encodings are identical

### Requirement: Redeal state inclusion
When the game's redeal limit is bounded, the encoding SHALL include the number of recycles still permitted, because it affects which moves are legal. When the redeal limit is unlimited, recycles remaining SHALL NOT be encoded (it never changes legality).

#### Scenario: Bounded redeal distinguishes otherwise-equal positions
- **WHEN** two positions have identical piles under a bounded redeal limit but different numbers of recycles remaining
- **THEN** their encodings differ

#### Scenario: Unlimited redeal ignores recycle count
- **WHEN** two positions have identical piles under an unlimited redeal limit but different recycle counts
- **THEN** their encodings are identical
