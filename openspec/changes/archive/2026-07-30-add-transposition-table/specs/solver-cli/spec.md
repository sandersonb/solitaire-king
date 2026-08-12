## ADDED Requirements

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
