## Context

The archived heuristic solver is a per-path DFS: it remembers only the current path's ancestors (for cycle avoidance) and forgets everything on backtrack, so it re-explores positions reached by different move orders. Klondike has heavy transposition, so this redundancy dominates runtime — the solver proves only ~10–15% of deals within 15s. This change adds a global transposition table (a closed set of proven-winless positions) so each distinct position is expanded once, and it makes proving a deal *unwinnable* possible. It reuses the existing `PositionKey` encoding and keeps all heuristics.

## Goals / Non-Goals

**Goals:**
- Global closed-set transposition table, bounded with sound eviction, on by default.
- Retain an independent, never-evicted on-path set as the termination guarantee.
- A three-way verdict: Solvable / Unwinnable (proven) / Inconclusive (budget hit).
- Compose with existing heuristics; disabling the table (and heuristics) reproduces the baseline.
- Report table stats; keep the library std-only.

**Non-Goals:**
- Disk persistence of the table, a `(seed,config)` results cache, and an endgame tablebase (future work — see below).
- Zobrist 64-bit keying (future optimization; we key on `PositionKey` bytes now).
- Parallel search.

## Decisions

### Two structures: on-path set (termination) + closed-set TT (speed)
The search keeps both:
- **On-path set** — a `HashSet<PositionKey>` of the current DFS ancestors, pushed on entry and popped on exit, **never evicted**. This alone guarantees termination (a finite state space with no repeated position on any single path).
- **Closed-set transposition table** — records a position only *after* its whole subtree is explored with no win (post-order "mark closed"). It holds exclusively **proven-winless** positions, so skipping one is always correct and *evicting* one is always safe (worst case we recompute it). This is what removes the cross-branch redundancy.

Per node: if in the TT → skip (winless); if in the on-path set → skip (cycle); otherwise push on-path, expand (heuristics/ordering unchanged), and on return with no win, pop on-path and insert into the TT. Chosen over a single "mark on entry" set because eviction of an on-path entry could otherwise reintroduce infinite loops — separating the two keeps termination independent of table capacity.

### Full caching, with graph-history interaction handled empirically
Combining a transposition table with cycle pruning creates the classic **GHI** problem: a position pruned as a cycle on one path could, in principle, be winnable on another, so caching it as winless risks missing a win (or a false unwinnable). A strictly GHI-safe rule (only cache when no cycle escaped the subtree to a shallower ancestor) was implemented first — but in Klondike, stock-recycle cycles escape upward pervasively, so it cached almost nothing (~144 entries in a 147k-node search) and gave no speedup. We therefore use **full caching** (cache every fully-explored, non-budget-cut winless position), the standard and effective approach, and mitigate GHI **empirically** via the differential validator: on positions that search to completion, table-on and table-off must reach the same verdict (guarded by tests). Reported wins remain unconditionally sound (every winning moveset is replay-verified); the residual risk is a rare false *unwinnable* on a deal whose whole space happens to exhaust within budget — acknowledged and monitored rather than eliminated. A rigorous GHI-safe scheme is possible future work.

### Bounded `HashSet` with clear-on-full eviction
The TT is a `HashSet<PositionKey>` capped at `max_table_entries`. Below the cap it never evicts; at the cap it clears (a generational reset) to stay bounded. This was chosen over a direct-mapped array after measurement: a direct-mapped cache (one key per `hash % capacity` bucket) sheds entries on *hash collision* far below capacity — at ~15–20% fill the birthday paradox produced tens of thousands of evictions (e.g. ~83k on a 750k-entry / 4M-cap run), each forgetting a proven-winless position that then had to be recomputed. The `HashSet` resolves collisions properly, so under the cap eviction is zero and every winless verdict is retained; it also uses less memory here (no large, mostly-empty bucket array). Trade-off: clear-on-full is crude for searches that truly exceed the cap (rare); an LRU/two-tier policy is a later tuning knob. Sound as before — only proven-winless positions are stored, so dropping any just costs recomputation.

### Verdict as an explicit enum
`SolveResult` gains `verdict: Verdict { Solvable, Unwinnable, Inconclusive }`. `Unwinnable` is set only when the root DFS returns with no win **and** no budget limit was hit (the reachable space was exhausted — valid even with eviction, since eviction never skips a position unsafely). `Inconclusive` when a node/time limit stopped it first. `solvable` stays as a convenience mirror of `verdict == Solvable`.

### Table on by default; baseline disables it
`SolveOptions` gains `transposition_table: bool` (default true) and `max_table_entries: usize` (default a few million). `SolveOptions::baseline()` sets `transposition_table: false` (plus heuristics off), so the archived per-path solver is reproducible for before/after measurement. The differential validator is extended so table-on vs table-off verdict agreement can be checked on completing positions.

### Memory accounting follows the table
Peak logical memory becomes the on-path set plus the live table occupancy (entries × key bytes). The portable metric we built now reflects the real driver of memory. OS RSS remains out of scope.

## Risks / Trade-offs

- **Eviction thrash on undersized tables** → Default capacity generous; expose `--max-table-entries`; document that a small table trades speed, never correctness. Stats (hits/evictions) make thrash visible.
- **`PositionKey` bytes are heavier than a 64-bit hash** → Accepted for v1 correctness/simplicity; Zobrist is a clean, isolated future optimization that keeps the same interface.
- **Proving unwinnable still bounded by budget** → Large reachable spaces won't exhaust within 15s; those stay Inconclusive. Honest and expected; the TT still raises solved+proven dramatically.
- **A closed-set marks winless correctly only under the current config** → The table is per-run and per-config; we never persist it, so no stale-entry hazard. (Persistence would require config/ruleset versioning — see below.)

## Open Questions / Future Work

- **Disk persistence** — deliberately deferred. The high-value TT entries (deep mid-game subtrees) are shuffle-specific and barely shared across deals, while the shared entries (endgames) are cheap to recompute; a dumped table is also config-specific and correctness-fragile. The principled durable-reuse paths are a small **`(seed, config) → verdict/moveset` results cache** and an **endgame tablebase** (precompute all positions with ≤ N cards off the foundations) — each a good standalone future change.
- **Zobrist keying** and **replacement policy** (LRU / two-tier / depth-preferred) — future optimizations measurable against this baseline.
- Default `max_table_entries` value — will pick a memory-reasonable default (e.g. a few million) at apply time, tunable via flag.
