## Why

The brute-force solver is correct but impractical: with no move ordering it wanders down dead-end lines and reports *inconclusive within budget* on most winnable deals. Before investing in a transposition table, we can make the solver genuinely useful by (a) **forcing provably-safe foundation auto-moves**, (b) **ordering the remaining moves** so promising lines are tried first, and (c) adding cheap **safe pruning**. Together these should let it actually find solutions to a large fraction of winnable deals within the existing budget — while the naive baseline stays available (behind flags) so the improvement is measurable.

## What Changes

- Add **safe foundation auto-moves as forced moves**: when a card is safe to send to its foundation — rank ≤ 2, or **both opposite-color foundations at rank ≥ r−1** — play it as the node's sole move (no branching). Provably sound (never removes a win); collapses large subtrees. On by default.
- Add **heuristic move ordering** (never removes moves, so completeness is preserved). Priority at each node:
  1. forced safe auto-moves (above);
  2. moves that reveal a face-down tableau card, tie-broken by the **larger** source stack;
  3. other productive builds (tableau↔tableau that unblock, waste→tableau, unsafe-but-useful foundation plays);
  4. stock draw — **penalized when the previous move was also a draw** (avoid spinning the stock without progress);
  5. **foundation→tableau — strictly last resort** (no hard cap; completeness preserved).
- Add **empty-column symmetry** pruning: when several tableau columns are empty, only consider moving a King to **one** of them (they are interchangeable). Safe; cuts King-move branching.
- Make the digging-direction tie-break (larger vs. smaller source stack) a **configurable knob** (default: larger), since both heuristics have merit and we want to measure.
- Expose all heuristics as **toggles** (default on; equivalence pruning stays default off) so the naive baseline can be reproduced and benchmarked, and extend the differential validator to confirm the heuristics don't change any solvable verdict.
- Surface heuristic **flags** in the `--solve` CLI and report a couple of extra stats (e.g. forced auto-moves played) to make the speed-up visible.

Non-goals (deferred): the transposition table / global memoization (the next solver), unwinnability *proofs*, minimal solutions, parallelism, and the experimental equivalence rule's promotion to default.

## Capabilities

### New Capabilities

<!-- None; this extends the existing solver capabilities. -->

### Modified Capabilities

- `solver-engine`: Adds forced safe auto-moves, heuristic move ordering, and empty-column symmetry pruning to the search — all preserving the "find a win if one exists within budget" guarantee.
- `solver-cli`: Adds flags to toggle the heuristics and choose the digging direction, and reports the new heuristic statistics.

## Impact

- **Library**: extends `src/solver/` — a new ordering/heuristics layer (`classify.rs`/a new `heuristics.rs`) and changes to `search.rs`'s move generation. Still std-only.
- **Public API**: `SolveOptions` gains fields (`safe_automoves`, `move_ordering`, `empty_column_symmetry`, `dig_larger_first`), all defaulting to the useful (heuristic-on) configuration; `SolveResult` may gain a stat or two. Existing callers using `SolveOptions::default()` get the improved solver automatically.
- **CLI**: `--solve` gains heuristic toggle flags; interactive play is unaffected.
- **Measurability**: disabling the new options reproduces the archived brute-force baseline, so before/after node/time comparisons are exact.
