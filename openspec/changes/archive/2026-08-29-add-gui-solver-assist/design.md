## Context

See `proposal.md` — Why. Constraints that shape this design:

- **The deployed GUI is single-threaded WebAssembly.** `wasm32-unknown-unknown`
  has no `std::thread`, and `std::time::Instant::now()` **panics** there. The
  current solver (`src/solver/search.rs`) calls `Instant::now()` unconditionally
  and runs a synchronous recursive DFS, so today it cannot run on the web build at
  all, and a 1000 ms synchronous solve would freeze the frame loop on any target.
- **The search is recursive**, not resumable: `Search::dfs` recurses and
  backtracks in one call. There is no pause/resume/step API.
- **The engine already has a transposition table** (`ClosedTable<K>`) that records
  proven-winless positions, but it is created fresh inside each `solve_state` call
  and never returned. Positions are keyed by `encode`/`PositionKey` (exact bytes)
  or `zobrist` (hash).
- The GUI loop has a monotonic clock via macroquad `get_time()` (works on web),
  an idle-able input model, and existing font + button + overlay rendering.

## Goals / Non-Goals

**Goals:**
- Run solvability checks from the GUI on native and web without freezing, capped
  at ~1000 ms of solver work per check.
- Preserve solver knowledge across checks and moves so re-checks (and post-undo
  checks) are cheap.
- A correct four-state indicator and a non-nagging unwinnable dialog whose status
  is per-position and survives undo/redo.

**Non-Goals:**
- No true multithreading or Web Workers (kept single-threaded/cooperative).
- No conversion of the recursive DFS into a fully resumable state machine.
- No solver *hints*/auto-play (the animation `enqueue_moves` hook exists for that
  later; out of scope here).
- No change to deal, rules, scoring, or the CLI solver's observable behavior.

## Decisions

### D1: Make solver timing WebAssembly-safe (don't depend on `Instant`)
Guard wall-clock use behind `cfg(not(target_arch = "wasm32"))`. On wasm the
elapsed field is reported as `Duration::ZERO` (or `None`) and the search is bounded
purely by the node budget; the time deadline is simply never armed. Native keeps
the existing time budget. This is the minimal change that unblocks the web build.
- *Alternative:* inject a clock trait/closure. Rejected as heavier than needed —
  the GUI already owns the only wall clock we need (`get_time()`), and it enforces
  the 1000 ms cap itself (D2).

### D2: Cooperative slicing via repeated node-bounded searches sharing one table
Because the DFS isn't resumable, we don't step *within* a search. Instead the GUI
pumps **many short node-bounded searches** of the current position, all sharing a
**persistent `ClosedTable`**. Each call restarts at the root, but every subtree the
previous calls proved winless is now in the table and is pruned at its root, so the
frontier advances each call — repeated bounded searches converge to the same
decisive verdict an unbounded search would reach. The GUI:
1. each frame, if a check is active, runs one search with a small node budget
   (e.g. ~40k nodes) reusing the table;
2. sums the real time spent (via `get_time()` deltas) and stops the check at
   ~1000 ms → `Inconclusive`, or earlier on a `Solvable`/`Unwinnable` verdict.
- *Why acceptable:* the re-walk cost per call is bounded by table hits (O(depth)
  to reach the frontier), and a check is capped anyway. This trades a little
  redundant work for zero solver changes to the DFS control flow.
- *Alternative:* native thread + wasm slicing. Rejected — two code paths, and the
  slicing path is required for web regardless, so use it everywhere.

### D3: Exact-byte keys for the GUI table (no false "unwinnable")
The unwinnable **dialog makes a definitive claim**, so the GUI check uses
`KeyStrategy::ExactBytes` (`PositionKey`). Zobrist's small collision chance could
prune a live branch and wrongly report unwinnable; exact keys cannot. Cost is more
memory per entry, acceptable for one game's table.

### D4: One reusable table per game, sound across positions
New engine entry point:
`solve_reusing(root, node_budget, options, &mut ClosedTable<PositionKey>) -> SolveResult`.
It runs the existing search but takes the table by reference instead of allocating
one. `ClosedTable` and `PositionKey` are re-exported from `lib.rs`. Reuse across
different positions in the same game is sound because a key encodes the *complete*
state (tableau, stock/waste, foundations, redeal count), so "winless" is a property
of the position, not the path that reached it. The table is created on new-deal and
dropped/replaced on new-deal. Table eviction only drops pruning opportunities
(safe: worst case a re-search), never changes correctness.

### D5: GUI status model (`src/gui/solver.rs`)
```
enum Status { Unknown, Checking, Solvable, Unwinnable, Inconclusive }
struct Assist {
    table: ClosedTable<PositionKey>,          // persistent knowledge
    decided: HashMap<PositionKey, Verdict>,   // fast status for revisits/undo
    status: Status,
    check: Option<ActiveCheck>,               // current position key + time spent
    last_activity: f64,                       // get_time() of last move/interaction
    dialog_open: bool,
    dismissed_streak: bool,                   // suppress dialog after Continue
}
```
- **On new deal:** reset `table`, `decided`, `dismissed_streak`; start a check of
  the opening position immediately (req 1).
- **On any state change (move/undo/redo):** record activity; compute the current
  `PositionKey`. If it's in `decided`, set `status` from it immediately (and, if
  `Unwinnable` and `!dismissed_streak`, open the dialog); else `status = Unknown`,
  abandon any in-flight check, and clear `dismissed_streak` when leaving an
  unwinnable status.
- **Each frame:** if `status == Checking`, pump one slice (D2). If idle
  (`now - last_activity > 3s`) and `status ∈ {Unknown, Inconclusive}` and not
  known-unwinnable, start a check (req 3).
- **On decisive result:** record in `decided`; update indicator; if `Unwinnable`
  and `!dismissed_streak`, open the dialog.

### D6: Undo-aware, non-nagging behavior (req 6, worked through)
Status is keyed to the *current position*, so:
- **Proven unwinnable → dialog → Continue:** `dismissed_streak = true`; play
  continues. Every forward move from an unwinnable position is also unwinnable, so
  successor checks hit the table and resolve to `Unwinnable` instantly — indicator
  stays "unwinnable", but no dialog (streak dismissed). Req 6's "don't run the
  solver again" is satisfied in spirit: known positions come from `decided`/table
  with no new search; genuinely new successors resolve in one tiny table-pruned
  call.
- **Undo to an earlier position:** look up its key. If `decided` says winnable/
  inconclusive, or it's unknown, clear the unwinnable status and (re-)check; this
  is where "undo can make it solvable again" is handled. `dismissed_streak` is
  cleared when status leaves `Unwinnable`.
- **Re-reaching a known-unwinnable position (redo):** resolved instantly from
  `decided`/table; no re-search; dialog only if the streak isn't already dismissed.

### D7: Indicator and dialog rendering
- Indicator: a small glyph in a top corner drawn with the bundled font — check
  mark (solvable), ✗ (unwinnable), hourglass (checking), ? (uncertain/unknown).
  Rendered every frame from `status`.
- Dialog: a modal overlay (dim + panel) reusing the existing rounded-rect/button
  drawing, with **Continue** and **New game** buttons wired through the existing
  pointer/button handling. While open it swallows board input.

## Risks / Trade-offs

- **Repeated-restart overhead** if the table doesn't prune enough within a check →
  the check simply ends `Inconclusive` at the 1000 ms cap; indicator shows "?".
  Mitigate with a tuned per-slice node budget. → acceptable.
- **Frame hitching** if a slice's node budget is too large → keep slices small
  (~tens of thousands of nodes) and measure; the cap is on cumulative solver time,
  not per frame. → tune during implementation.
- **Table memory growth** over a long game → the table is bounded with sound
  eviction; eviction only costs re-work, never correctness. Use a generous cap.
- **Exact-key memory** larger than Zobrist → one game's worth; acceptable for
  correctness of the unwinnable claim.
- **`decided` cache staleness** — impossible within a game: keys are complete state
  encodings and verdicts for a fixed deal are immutable; the cache resets on
  new-deal. → safe.

## Migration Plan

1. Solver: make timing wasm-safe (D1); add `solve_reusing` taking `&mut
   ClosedTable<PositionKey>` (D4); re-export `ClosedTable`/`PositionKey`; unit
   tests (reuse equals one-shot verdict; converges under repeated small budgets;
   builds and runs under a node-only budget with no `Instant`). 2. GUI `solver.rs`
   Assist state machine + slice pump (D2/D5). 3. Indicator + dialog rendering
   (D7). 4. Wire activity/reset on move/undo/redo/new-game and dialog buttons
   (D5/D6). 5. Verify native + wasm builds, tests, clippy.
Rollback is clean: the feature is additive and GUI-local; not starting checks
disables it, and the solver entry point is new (existing `solve`/`solve_state`
unchanged).

## Open Questions

- Per-slice node budget and the exact idle threshold are tuning values, not
  spec-level decisions; they can be adjusted during implementation without
  changing the specs or the task breakdown.
