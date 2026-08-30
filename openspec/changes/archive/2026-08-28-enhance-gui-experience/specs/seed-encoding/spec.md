## Purpose

Provides a reversible, human-friendly encoding of the 64-bit deal seed so players
can read, remember, speak, and share a deal by a pronounceable string instead of a
bare `u64`, while keeping every existing raw-`u64` seed working.

## ADDED Requirements

### Requirement: Pronounceable seed encoding
The library SHALL provide a function that encodes any 64-bit deal seed as a
pronounceable string using the proquint scheme (alternating consonant/vowel
syllables), so the same `u64` always produces the same string and the string is
speakable and memorable. The encoding SHALL be total (defined for every `u64`).

#### Scenario: Deterministic encoding
- **WHEN** the same `u64` seed is encoded twice
- **THEN** both calls produce the identical string

#### Scenario: Distinct seeds encode distinctly
- **WHEN** two different `u64` seeds are encoded
- **THEN** they produce different strings

### Requirement: Seed decoding with raw fallback
The library SHALL provide a function that decodes a seed string back to the exact
`u64` it was encoded from, and SHALL also accept a raw decimal `u64` string so
that seeds recorded before this encoding existed still resolve. Decoding SHALL be
case-insensitive and SHALL ignore group separators. An input that is neither a
valid encoded seed nor a valid `u64` SHALL be rejected rather than silently
producing a wrong deal.

#### Scenario: Round trip
- **WHEN** a `u64` is encoded and the resulting string is decoded
- **THEN** the decoded value equals the original `u64`

#### Scenario: Raw u64 still accepted
- **WHEN** a plain decimal `u64` string is decoded
- **THEN** it resolves to that same `u64` value

#### Scenario: Invalid input is rejected
- **WHEN** a string that is neither a valid encoded seed nor a valid `u64` is decoded
- **THEN** decoding fails (returns no value) rather than resolving to an arbitrary seed
