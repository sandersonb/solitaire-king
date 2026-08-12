# solver-cli Specification

## Purpose

Defines the command-line surface for running the brute-force solver: a `--solve` mode that reuses play-mode game configuration, budget flags, and a readable solver result report.

## Requirements

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

### Requirement: Transposition-table flags
The `--solve` mode SHALL expose a flag to disable the transposition table and a flag to set its maximum capacity (in entries), each with a sensible default (table on). Disabling the table SHALL be included in the `--baseline` convenience flag.

#### Scenario: Table on by default
- **WHEN** `--solve` is run without table flags
- **THEN** the transposition table is active

#### Scenario: Capacity is configurable
- **WHEN** the user sets a maximum table entry count
- **THEN** the table is bounded to that capacity and evicts beyond it

#### Scenario: Baseline disables the table
- **WHEN** `--solve --baseline` is run
- **THEN** the transposition table is disabled along with the other heuristics

### Requirement: Flags grouped by concern in help
The CLI `--help` SHALL group flags under labeled headings by the concern they belong to — game configuration, solver search budget, solver heuristics, and transposition table — so it is clear which algorithm/layer each flag affects rather than presenting one flat list.

#### Scenario: Help groups the flags
- **WHEN** the user runs the binary with `--help`
- **THEN** solver search, heuristic, and transposition-table flags each appear under their own labeled heading, distinct from the game-configuration flags

### Requirement: Verdict and table stats in the report
The `--solve` report SHALL clearly distinguish the three verdicts — SOLVABLE, UNWINNABLE (proven), and INCONCLUSIVE (budget/limit hit) — and SHALL include the transposition-table statistics (entries, hits, evictions) alongside the existing figures.

#### Scenario: Proven-unwinnable is shown distinctly
- **WHEN** the solver proves a deal unwinnable
- **THEN** the report states UNWINNABLE (proven), distinct from INCONCLUSIVE

#### Scenario: Report includes table stats
- **WHEN** the solver finishes with the table enabled
- **THEN** the report shows the table entries, hits, and evictions

### Requirement: Key-strategy flag
The `--solve` mode SHALL expose a flag to select exact byte keys instead of the default Zobrist hash (for reproducing the collision-free reference run), grouped under the transposition-table flags. The default (flag absent) SHALL be Zobrist.

#### Scenario: Default uses Zobrist
- **WHEN** `--solve` is run without the key-strategy flag
- **THEN** the solver uses Zobrist keys

#### Scenario: Exact keys selectable from the CLI
- **WHEN** the user passes the exact-keys flag
- **THEN** the solver uses the byte encoding as its key
