## ADDED Requirements

### Requirement: Heuristic toggle flags
The `--solve` mode SHALL expose flags to control the search heuristics: disabling safe foundation auto-moves, disabling move ordering, disabling empty-column symmetry pruning, and choosing the digging direction (prefer larger or smaller source column). It SHALL also provide a single `--baseline` convenience flag that disables all heuristics at once. All heuristics SHALL default to on (the useful configuration); disabling them SHALL reproduce the naive brute-force baseline for comparison.

#### Scenario: Heuristics on by default
- **WHEN** `--solve` is run without heuristic flags
- **THEN** safe auto-moves, move ordering, and empty-column symmetry pruning are all active

#### Scenario: Baseline reproduction
- **WHEN** the heuristics are all disabled via flags
- **THEN** the solver behaves as the naive baseline (only no-op and cycle pruning remain)

#### Scenario: Baseline convenience flag
- **WHEN** `--solve --baseline` is run
- **THEN** all heuristics are disabled in one flag, equivalent to disabling each individually

### Requirement: Heuristic statistics in the report
The `--solve` report SHALL include at least one statistic that reflects the heuristics' effect (for example, the number of forced safe auto-moves played), alongside the existing nodes/time/memory figures.

#### Scenario: Report shows heuristic effect
- **WHEN** the solver finishes with heuristics enabled
- **THEN** the report includes a heuristic-related statistic in addition to nodes, time, and memory
