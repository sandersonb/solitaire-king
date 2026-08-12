## Why

The solver is now throughput-bound: at ~55–60k nodes/s it only covers ~900k nodes in 15s, so the hard deals stay inconclusive. Two constant-factor wins address the dominant per-node costs: **make/unmake** eliminates the full `GameState` clone (≈13 heap allocations) performed for every child, and **Zobrist hashing** replaces the ~60-byte `PositionKey` (allocated + SipHashed per node) with a compact 128-bit key that is smaller to store and faster to hash/compare. Together they should multiply the node rate, letting the transposition table cover more of each deal within the same budget.

## What Changes

- Add **reversible move application** to the rules engine: alongside `apply_move`, an undoable apply returns a small `Copy` undo token, and `undo_move` reverses a move to the exact prior state (restoring piles, auto-flips, score, and recycle count) with **zero heap allocation**.
- Rework the solver's search to **make/unmake in place**: it mutates a single `GameState` down each branch and undoes on backtrack, instead of cloning a child per candidate move.
- Add **128-bit Zobrist position hashing**: a deterministic hash over card positions (canonical exactly as the byte encoding — positional-only, foundations by suit, recycles-remaining when the redeal is bounded). The solver keys its transposition table and on-path set on this hash by default.
- Keep the exact byte-key (`PositionKey`) as a **selectable key strategy** (`SolveOptions`), and extend the **differential validator** to confirm Zobrist-keyed and byte-keyed runs reach the same verdict on positions that search to completion — a soundness backstop for the (astronomically small at 128-bit) collision risk.
- Surface the key strategy via a CLI flag (e.g. `--exact-keys`) for reproducing the collision-free reference run.

Non-goals (deferred): *incremental* Zobrist updates threaded through make/unmake (from-scratch per-node hashing is chosen for simplicity/safety; incremental is a future micro-opt), migrating the interactive CLI's undo from snapshots to make/unmake, parallel search, and any table persistence.

## Capabilities

### New Capabilities

<!-- None; this extends existing capabilities. -->

### Modified Capabilities

- `klondike-rules-engine`: Adds reversible (undoable) move application — an undo token and `undo_move` that exactly reverses any legal move.
- `solver-state-encoding`: Adds 128-bit Zobrist position hashing with the same canonicalization as the byte encoding.
- `solver-engine`: Search uses make/unmake (no per-child clone) and a selectable position key (Zobrist by default, exact bytes for validation); the differential validator covers the key strategy.
- `solver-cli`: Adds a key-strategy flag.

## Impact

- **Library**: `src/model/rules.rs` gains `apply_undoable`/`undo_move` + an `Undo` token (and a few pile helpers, e.g. flip-down); `src/solver/` gains a `zobrist.rs` (feature table + hashing) and reworks `search.rs` to mutate-in-place and to be generic over the key type. Still std-only.
- **Public API**: `SolveOptions` gains a `key` strategy field (default Zobrist-128); `apply_undoable`/`undo_move`/`Undo` are exported. Existing `GameState::apply` and the interactive CLI are unchanged.
- **Performance**: the hot loop drops per-child allocation and shrinks keys from ~60 B to 16 B — expected to raise node rate substantially and pack far more positions per MB (so `--max-table-entries` buys more).
- **Soundness**: guarded by property tests (apply→undo is the identity) and the Zobrist-vs-exact differential validator; a reported win remains replay-verified regardless.
