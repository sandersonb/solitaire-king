## 1. Reversible moves in the rules engine (klondike-rules-engine)

- [x] 1.1 Add pile helpers needed for reversal (e.g. `TableauColumn::flip_top_down`, a stock drain-top-first) without disturbing existing behavior
- [x] 1.2 Define a `Copy` `Undo { drawn, flipped, prev_score, prev_recycles }` token
- [x] 1.3 Implement `apply_undoable(state, mv) -> Result<Undo, MoveError>` (same effects as `apply_move`, returning the token); reimplement `apply_move` to call it and drop the token
- [x] 1.4 Implement `undo_move(state, mv, undo)` reversing each move variant exactly (piles, auto-flip via `flipped`, score and recycle count via the token), with no heap allocation
- [x] 1.5 Export `Undo`, `apply_undoable`, `undo_move` from the crate

## 2. Rules-engine equivalence tests

- [x] 2.1 Property test: for a set of seeds, at each reachable state and for every legal move, `apply_undoable` then `undo_move` restores the exact prior state (byte encoding equal, score/recycles equal)
- [x] 2.2 Test: `apply_undoable` yields the same state as cloning and `apply_move` (make == clone-and-apply) across the move variants, including auto-flip and recycle

## 3. Zobrist hashing (solver-state-encoding)

- [x] 3.1 Add `src/solver/zobrist.rs`: a deterministic feature table (`Z_card[card][pile][depth][face]`, `Z_foundation[suit][rank]`, `Z_recycles[remaining]`) generated once with the in-crate PRNG and a fixed seed
- [x] 3.2 Implement `zobrist(state) -> u128`: XOR of tableau/stock/waste card features and per-suit foundation features, plus recycles-remaining when the redeal is bounded; foundation cards counted only via the foundation feature
- [x] 3.3 Unit-test: determinism; score/time excluded; foundation-slot interchangeability; and that Zobrist equality matches byte-encoding equality across many positions from a real search

## 4. Selectable key + generic search (solver-engine)

- [x] 4.1 Add `KeyStrategy { Zobrist, ExactBytes }` to `SolveOptions` (default `Zobrist`); thread it through
- [x] 4.2 Make the search generic over the key type `K: Hash + Eq + Clone`; instantiate with `u128` (Zobrist) or `PositionKey` (bytes) in `solve_state`
- [x] 4.3 Rework `dfs` to make/unmake in place (`apply_undoable`/`undo_move`) on one `&mut GameState` instead of cloning children
- [x] 4.4 Split the no-op check: structural part on the pre-move state, exposed-card part after applying; undo and skip when a no-op
- [x] 4.5 Keep verdict/stat/memory reporting intact (key bytes now reflect the chosen strategy)

## 5. Validation (solver-engine)

- [x] 5.1 Add `validate_key_strategy(seeds, config, budget)` comparing Zobrist vs exact-byte verdicts; export it
- [x] 5.2 Integration test: on completing positions (near-won, deadlock, the stock deal), Zobrist and exact-byte keys give identical verdicts
- [x] 5.3 Integration test: make/unmake search matches the archived behavior — solvable/unwinnable/inconclusive verdicts unchanged on the existing constructed positions; replayable winning moveset still valid
- [x] 5.4 Re-confirm all existing solver, heuristic, and table tests pass

## 6. CLI (solver-cli)

- [x] 6.1 Add an `--exact-keys` flag (under the Transposition-table heading) mapping to `KeyStrategy::ExactBytes`; default Zobrist
- [x] 6.2 Unit-test the flag→`SolveOptions` mapping

## 7. Verification & measurement

- [x] 7.1 Manual run: on seeds 21/22/23/10231, compare node rate and reach before vs after (expect higher nodes/s and more positions per MB); confirm `--exact-keys` agrees on any that complete
- [x] 7.2 Run `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` clean
