## ADDED Requirements

### Requirement: Key-strategy flag
The `--solve` mode SHALL expose a flag to select exact byte keys instead of the default Zobrist hash (for reproducing the collision-free reference run), grouped under the transposition-table flags. The default (flag absent) SHALL be Zobrist.

#### Scenario: Default uses Zobrist
- **WHEN** `--solve` is run without the key-strategy flag
- **THEN** the solver uses Zobrist keys

#### Scenario: Exact keys selectable from the CLI
- **WHEN** the user passes the exact-keys flag
- **THEN** the solver uses the byte encoding as its key
