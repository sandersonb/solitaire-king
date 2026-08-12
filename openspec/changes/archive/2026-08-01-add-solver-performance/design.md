## Context

Profiling by inference: the solver's hot loop clones a full `GameState` per candidate move (≈13 `Vec` allocations) and, per node, builds a ~60-byte `PositionKey` and SipHashes it. At ~55–60k nodes/s the hard deals never exhaust their space in 15s. Two constant-factor optimizations target these costs directly — make/unmake (kill the clone) and Zobrist keys (shrink/speed the key) — without changing what the search explores or concludes. The transposition table, on-path guard, heuristics, and verdicts all stay as archived; this change only makes each node cheaper and each key smaller.

## Goals / Non-Goals

**Goals:**
- Reversible move application in the rules engine (zero-alloc undo token) and an in-place make/unmake search.
- Deterministic 128-bit Zobrist hashing, canonical exactly as the byte encoding.
- Selectable key strategy (Zobrist default, exact bytes as a collision-free reference) with differential validation.
- Pure optimization: identical nodes/verdict to the archived solver; strong equivalence tests.

**Non-Goals:**
- *Incremental* Zobrist threaded through make/unmake (from-scratch per node is simpler and safe; incremental is a future micro-opt).
- Migrating the interactive CLI's snapshot-undo to make/unmake; parallelism; table persistence.

## Decisions

### Reversible moves via a `Copy` undo token
Add `apply_undoable(state, mv) -> Result<Undo, MoveError>` and `undo_move(state, mv, undo)` to the rules engine; keep `apply_move` (it calls `apply_undoable` and drops the token). `Undo` is a small `Copy` struct — `{ drawn: u8, flipped: bool, prev_score: Score, prev_recycles: u32 }` — capturing the only facts a reversal needs beyond the move itself: how many cards a draw moved, whether an auto-flip fired, and the prior score/recycle count. Reversal moves cards between existing piles with `drain`/`extend`/`push_run` (no new `Vec`), un-flips the exposed card when `flipped`, and restores `score`/`recycles_done`. A couple of pile helpers are added (e.g. flip-top-down, stock drain-top-first). Rationale: a token-based make/unmake is the standard zero-allocation approach and keeps rules logic in one place; the token is tiny and lives on the stack. Alternative — partial-pile snapshots — still allocates and was rejected.

### In-place search
`dfs` takes `&mut GameState`: it `apply_undoable`s a candidate, recurses, then `undo_move`s on return — one state threaded down each branch. The no-op check is split so its structural part (reveals-no-face-down, interchangeable host) runs on the pre-move state and the "exposed card has no move" part runs after applying; if it's a no-op we simply undo and skip. The winning line is still captured by cloning the `Vec<Move>` path at the win (cheap, rare). Rationale: removes the dominant allocation; behavior is identical because apply→undo is the exact inverse (guaranteed by tests).

### From-scratch 128-bit Zobrist, not incremental
A fixed random table assigns a `u128` to each feature: `Z_card[card][pile][depth][face]` for tableau/stock/waste placements and `Z_foundation[suit][top_rank]` for foundations; a bounded redeal adds a `Z_recycles[remaining]` term. A position's hash is the XOR of its features, recomputed per node from the (mutated) state — an O(52) walk with no allocation, producing a 16-byte key. We deliberately do **not** thread incremental XOR updates through make/unmake: the from-scratch walk is cheap relative to `legal_moves`/apply, and incremental updates that must exactly mirror every move (including auto-flips and recycles) are error-prone for little marginal gain. The table is generated once (lazily, via the in-crate PRNG with a fixed seed) so hashing is deterministic. Cards on foundations are represented only by the foundation feature (not double-counted). This matches the byte encoding's canonicalization: positional-only, suit-canonical foundations, bounded-redeal recycles.

### Selectable key type via a generic search
`SolveOptions` gains `key: KeyStrategy { Zobrist, ExactBytes }` (default `Zobrist`). The search is generic over the key type `K: Hash + Eq + Clone`; `solve_state` monomorphizes it with `u128`-backed Zobrist keys or `PositionKey` byte keys, so there is no runtime dispatch cost. Both modes compute the key from the current state per node (make/unmake used in both). `ExactBytes` is the collision-free reference; `validate_key_strategy(seeds, …)` compares the two verdicts on completing positions. Rationale: keeping the exact path is a cheap, high-value soundness backstop for the (128-bit ⇒ ~1e-24) collision risk, and reuses the existing differential-validator pattern.

### 128-bit width
Keys are `u128` (16 B) — still 4× smaller than the byte key, with collision probability negligible even at tens of millions of entries, which protects the *proven-unwinnable* verdict from a hash artifact. (64-bit was considered and rejected here to keep that claim robust.)

## Risks / Trade-offs

- **Make/unmake asymmetry (a subtle undo bug)** → Property tests: for many random states × every legal move, `apply_undoable` then `undo_move` reproduces the exact original (byte encoding equal); and make equals clone-and-apply. A bug shows up immediately.
- **Zobrist collision corrupts a verdict** → 128-bit makes this ~1e-24; additionally the exact-byte differential validator would catch any real divergence on completing positions, and reported wins are always replay-verified.
- **From-scratch hashing keeps the O(52) walk** → Accepted; it's cheap and allocation-free, and the big wins (no clone, 16-byte keys, cheap hashing) still land. Incremental remains available as a future step.
- **More positions fit per MB → searches may run longer/deeper before the cap** → Desirable; `--max-table-entries` still bounds it and clear-on-full still applies.

## Open Questions

- Default `KeyStrategy` is `Zobrist`; `--exact-keys` selects the reference. (Settled.)
- Whether to later add incremental Zobrist — deferred; will revisit if per-node hashing shows up as a bottleneck after make/unmake lands.
