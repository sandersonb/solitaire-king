## Why

The heuristic solver finds *easy* wins in milliseconds but proves only ~10–15% of deals before exhausting its 15s budget — not because the rest are unwinnable, but because per-path DFS re-explores the same position astronomically many times when it is reachable by many move orders (Klondike is rife with such transpositions). A **transposition table** — a global cache of already-resolved positions — collapses that redundancy so each distinct position is expanded once. This is the single biggest lever left, and it unlocks a genuinely new capability: **proving a deal unwinnable**.

## What Changes

- Add a **global transposition table (closed set)**: positions that have been fully explored and found to contain no win are recorded; if the search reaches a position already in the table, it skips it entirely (no matter the path). Keyed by the existing `PositionKey` encoding.
- Keep a separate **on-path (ancestor) set** — never evicted — as the termination guarantee (prevents cycles), independent of the evictable table. The table only ever holds *proven-winless* positions, so skipping or evicting an entry is always sound.
- Make the table **bounded with eviction**: a fixed-capacity, direct-mapped structure (hash bucket → at most one key; inserting evicts the previous occupant of that bucket). Eviction only costs re-exploration, never correctness. Capacity is configurable.
- Add a **proven-unwinnable verdict**: if the search exhausts the reachable state space within budget without a win, report **UNWINNABLE (proven)** — distinct from **INCONCLUSIVE** (a node/time budget was hit first).
- Keep all existing **heuristics** (safe auto-moves, ordering, symmetry): they compose with the table, finding wins sooner and reducing how many distinct positions are expanded.
- The table is **on by default** (it is strictly better for the goal); a flag disables it to recover the archived per-path behavior for comparison.
- Report **table statistics** (entries, hits, evictions) and the sharpened verdict via `--solve`.

Non-goals (deferred): persisting the table to disk, a results cache (`seed → verdict`), an endgame tablebase, Zobrist 64-bit keying, and parallel search. These are noted as future work in the design.

## Capabilities

### New Capabilities

<!-- None; this extends the existing solver capabilities. -->

### Modified Capabilities

- `solver-engine`: Adds the global transposition table (closed set) with bounded eviction alongside the retained on-path cycle guard, and the proven-unwinnable verdict; preserves soundness and the "find a win if one exists within budget" guarantee.
- `solver-cli`: Adds flags for the table (capacity, disable) and reports the UNWINNABLE verdict plus table statistics.

## Impact

- **Library**: extends `src/solver/` with a `table.rs` (bounded closed-set) and changes to `search.rs` (mark-closed on exit, probe on entry, retained on-path set). Reuses `PositionKey` unchanged. Still std-only.
- **Public API**: `SolveOptions` gains `transposition_table: bool` (default true) and `max_table_entries: usize`; `SolveResult` gains a `Verdict` (Solvable / Unwinnable / Inconclusive) plus table stats (entries, hits, evictions). `SolveOptions::baseline()` disables the table (per-path only).
- **CLI**: `--solve` gains `--no-transposition` and `--max-table-entries`; the report distinguishes proven-unwinnable and shows table stats.
- **Memory profile shifts**: memory now grows with distinct positions held (bounded by the table capacity) rather than path depth — the portable logical-memory metric becomes meaningful and is driven by the table size.
