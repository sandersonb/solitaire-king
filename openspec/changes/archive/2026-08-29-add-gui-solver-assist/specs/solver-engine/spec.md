## ADDED Requirements

### Requirement: Platform-independent time budgeting
The solver SHALL NOT depend on a wall clock that is unavailable on the deployed
target: on a platform without a usable monotonic clock (the WebAssembly build),
the engine SHALL still run, bounding the search by the node budget, rather than
failing or panicking. On platforms with a monotonic clock, the elapsed-time
budget SHALL continue to apply as before. The node budget SHALL be honored on all
platforms.

#### Scenario: Runs on a platform without a monotonic clock
- **WHEN** the solver runs on the WebAssembly target with a node budget and no time budget
- **THEN** it completes the bounded search without attempting an unsupported clock call

#### Scenario: Node budget bounds the search everywhere
- **WHEN** a node budget is set on any platform
- **THEN** the search stops once that many nodes have been expanded if no win is found first

#### Scenario: Time budget still applies natively
- **WHEN** the solver runs on a native platform with an elapsed-time budget
- **THEN** it stops when that time is reached, as before

### Requirement: Reusable transposition table across searches
The solver SHALL provide an entry point that accepts a caller-owned transposition
table and a node budget, runs a bounded search from a given position, and returns
the result while leaving the table populated with the proven-winless positions it
found. Passing the same table into a later bounded search (from the same or a
different reachable position) SHALL let that search skip positions already proven
winless, so repeated bounded searches make monotonic progress toward a decisive
verdict. Reuse SHALL be sound: a position proven winless remains winless whenever
it is reached again, because positions are keyed by their complete encoded state.

#### Scenario: A shared table carries knowledge between searches
- **WHEN** a bounded search populates a caller-owned table and a second bounded search is run with that same table
- **THEN** the second search skips positions the first proved winless

#### Scenario: Repeated bounded searches converge
- **WHEN** a position is searched by repeated node-bounded calls that share one table
- **THEN** the calls collectively reach the same decisive verdict (solvable or proven unwinnable) that an unbounded search would, without redoing proven-winless subtrees

#### Scenario: Reuse preserves the standalone result
- **WHEN** a single search is run with a fresh caller-owned table and the full budget
- **THEN** its verdict matches the existing one-shot solver for that position and budget
