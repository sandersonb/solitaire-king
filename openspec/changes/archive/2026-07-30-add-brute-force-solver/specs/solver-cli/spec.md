## ADDED Requirements

### Requirement: Solve mode
The CLI SHALL provide a `--solve` mode that runs the solver on a deal instead of starting the interactive game. It SHALL accept the same seed and game-configuration flags as play mode (seed, draw mode, redeal limit) so the solved deal matches a playable one.

#### Scenario: Solve a specific seed
- **WHEN** the binary is run with `--solve --seed 42`
- **THEN** it runs the solver on the deal for seed 42 and prints a result instead of entering interactive play

#### Scenario: Solve respects game configuration
- **WHEN** `--solve` is combined with `--draw 1` or `--redeal N`
- **THEN** the solver uses that configuration for the deal

### Requirement: Solver budget flags
The `--solve` mode SHALL expose the search budget as flags — a maximum node count and a maximum time — each with sensible defaults when omitted.

#### Scenario: Budget flags are honored
- **WHEN** the user passes a maximum-time (or maximum-node) budget flag
- **THEN** the solver stops when that limit is reached and the output reflects it

### Requirement: Solver result output
The `--solve` mode SHALL print a readable report: whether the deal was solved, the winning moveset when solved, the elapsed time, and the peak logical memory. When the budget is exhausted without a win, it SHALL clearly state the result is inconclusive.

#### Scenario: Solved output
- **WHEN** the solver finds a win
- **THEN** the output shows solvable, the winning moveset, the elapsed time, and the peak memory

#### Scenario: Inconclusive output
- **WHEN** the budget is exhausted with no win found
- **THEN** the output states the outcome is inconclusive (not proven unwinnable) and shows the statistics gathered
