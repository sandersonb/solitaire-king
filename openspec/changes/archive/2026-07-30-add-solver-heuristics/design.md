## Context

The archived brute-force solver (`solver-engine`, `solver-state-encoding`, `solver-cli`) is a correct per-path DFS with no move ordering, so it reports *inconclusive within budget* on most winnable deals. This change adds heuristics that make it find real solutions fast, while keeping every guarantee (a win is found if reachable within budget) and keeping the naive baseline reproducible for measurement. It deliberately stops short of a transposition table — that is the next solver.

## Goals / Non-Goals

**Goals:**
- Force provably-safe foundation auto-moves (opposite-color rule) to collapse subtrees.
- Order the remaining moves so promising lines are explored first, without dropping any legal move (completeness preserved).
- Add empty-column symmetry pruning (safe).
- Keep all heuristics toggleable (default on) so the baseline is reproducible and the win is measurable; extend the differential validator to prove verdict-preservation.

**Non-Goals:**
- Transposition table / global memoization (next change).
- Unwinnability proofs, minimal solutions, parallelism.
- Promoting the experimental equivalence rule to default.

## Decisions

### Safe auto-moves: the opposite-color rule, forced
A foundation move is **safe** iff the card's rank ≤ 2, or both **opposite-color** foundations are at rank ≥ r−1. Rationale: a card of rank r can only ever be needed in the tableau to host a rank-(r−1) card of the opposite color; if both opposite-color foundations have reached r−1, both such cards are already home and can never need a tableau host, so the card has no remaining tableau use — sending it up loses nothing. Aces/2s are unconditionally safe (a 2 only ever hosts an Ace, and Aces go straight up). This is a strict superset of the "min of all four + 1" rule (it also fires when foundations are uneven) and is provably sound. When a safe move exists at a node, we play it as the *only* child (forced), which collapses large subtrees; remaining (unsafe) foundation moves stay ordinary branch candidates. Chosen over the global-min rule for more collapsing at equal safety.

### Move ordering: a total priority, never a filter
Ordering reorders candidates; it never removes them, so completeness is untouched (a win reachable before is still reachable, just found sooner). Priority: (1) forced safe auto-move; (2) reveals a face-down card (tie-break: digging direction); (3) other productive builds / waste plays / unsafe-but-useful foundation plays; (4) stock draw, penalized if the previous move was also a draw; (5) foundation→tableau, last. The draw penalty needs the previous move, and the F→T "last resort" is pure ordering — **no hard cap**, so we never abort a line that genuinely needs several foundation pulls (completeness preserved; we rely on the budget). Implemented as a sort key over `Move`, computed from the pre-state (and the previous move for the draw penalty).

### Empty-column symmetry: prune at move generation
When ≥2 columns are empty, generate a King-to-empty move for only one empty column. Empty columns are interchangeable, so the others are symmetric — sound. Cheaper and simpler than canonicalizing empty columns inside the encoding (which we still leave for later). Applied during candidate generation, before ordering.

### Forced-move / ordering / pruning as `SolveOptions`, default on
`SolveOptions` gains `safe_automoves`, `move_ordering`, `empty_column_symmetry` (default true) and `dig_larger_first` (default true); `no_op_pruning` stays true, `equivalence_pruning` stays false. So `SolveOptions::default()` is now the *useful* solver, and disabling the new flags reproduces the archived baseline exactly — enabling precise before/after benchmarking. The differential validator is extended so each heuristic can be toggled and verdict-preservation asserted across seeds.

### Search integration
`search.rs`'s per-node move step becomes: if `safe_automoves` and a safe move exists → single forced child; else generate `legal_moves`, apply `empty_column_symmetry` and no-op/equivalence filters, then (if `move_ordering`) sort by the priority key, then iterate as today (clone+apply, per-path cycle check, recurse, stop at first win). The safe-move check and ordering key live in a new `heuristics.rs`; `SolveResult` gains `forced_automoves` (count) for the report.

## Risks / Trade-offs

- **A heuristic accidentally drops a win** → Only empty-column symmetry and safe auto-moves *remove* branches, and both are provably sound; ordering never removes moves. All three are covered by the extended differential validator across many seeds (verdict must match baseline).
- **Ordering helps winnable deals but not hard/unwinnable ones** → Expected: without memoization, ordering can't stop re-exploration on lines that don't terminate in a win. Documented; the transposition table is the next change. The budget still bounds these.
- **Digging-direction heuristic is unproven either way** → It's a tie-break knob (never affects correctness); default larger, invertible, measurable.
- **Draw penalty could over-delay necessary draws** → It only reorders; all draws remain reachable, so no win is lost; worst case is minor extra exploration.

## Open Questions

- Whether to also expose a `--baseline` convenience flag that flips all heuristics off at once (nice-to-have; can add during apply).
- Which small set of seeds to use for the "heuristics find a win the baseline could not, within the same budget" benchmark test — will pick a few empirically at apply time.
