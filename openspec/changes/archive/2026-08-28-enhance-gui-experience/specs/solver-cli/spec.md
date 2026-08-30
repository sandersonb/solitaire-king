## MODIFIED Requirements

### Requirement: Solve mode
The CLI SHALL provide a `--solve` mode that runs the solver on a deal instead of starting the interactive game. It SHALL accept the same seed and game-configuration flags as play mode (seed, draw mode, redeal limit) so the solved deal matches a playable one; the seed SHALL accept either a pronounceable seed string or a raw `u64`. The solver report SHALL identify the deal by its pronounceable seed string.

#### Scenario: Solve a specific seed
- **WHEN** the binary is run with `--solve --seed 42`
- **THEN** it runs the solver on the deal for seed 42 and prints a result instead of entering interactive play

#### Scenario: Solve seed accepts either form
- **WHEN** `--solve` is given a seed as a pronounceable string or the equivalent raw `u64`
- **THEN** the solver runs on the identical deal and the report shows the pronounceable seed string

#### Scenario: Solve respects game configuration
- **WHEN** `--solve` is combined with `--draw 1` or `--redeal N`
- **THEN** the solver uses that configuration for the deal
