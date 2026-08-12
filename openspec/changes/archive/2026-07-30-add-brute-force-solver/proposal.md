## Why

The model and CLI can play Klondike, but nothing can *solve* it. This change adds the first automatic solver — a deliberately naive brute-force search — to establish a correct, well-tested baseline (and the vocabulary and infrastructure) that later, faster solvers can be measured against. It also forces us to define two concepts we will reuse everywhere: **no-op moves** and **equivalent moves**, plus a **compact state encoding** for cycle detection.

## What Changes

- Add a **`solver` module** (library, std-only) implementing a depth-first brute-force search that, for a given deal, reports:
  1. **whether the deal is winnable** (within the search budget),
  2. the **first winning moveset** found (need not be minimal), and
  3. the **elapsed time** and **logical memory consumed** to reach the result.
- Add a **compact state encoding** ("dump") that canonically represents a game position as bytes, used to detect **cycles** — if the current search path revisits a state already on it, that branch is discarded.
- Define and implement **no-op move** classification (a provably-safe reduction): a tableau→tableau move that reveals no face-down card, lands on an interchangeable host, and exposes a source card with no legal move (e.g. King on empty → another empty; a red 2 between two black 3s when neither 3 can move). No-op moves are skipped. Stock draws, foundation moves, and any move that reveals a card are never no-ops.
- Define **equivalent move** classification (interchangeable destinations for a freshly available card) and implement it **behind an off-by-default toggle**, since its soundness is unproven; include a **differential validation harness** that runs the search with and without a given pruning rule to confirm win-findability is unchanged (a way to "scientifically evaluate" the hypothesis).
- Add **budgeting** (`--max-nodes`, `--max-time`) so the exponential search terminates and reports partial/inconclusive results cleanly.
- Add a **CLI `--solve` mode**: `klondike --solve [--seed N] [budget flags]` runs the solver on the deal and prints winnable?/example-moveset/time/memory.

Non-goals (deferred): transposition tables / global memoization and any performance-oriented solver (explicitly a later change), minimal/optimal solutions, unwinnable *proofs* (we report "no win within budget = inconclusive"), parallel search, and save/replay formats.

## Capabilities

### New Capabilities

- `solver-state-encoding`: A compact, canonical byte encoding of a `GameState` position (tableau with face-up flags, stock, waste, suit-canonical foundations, and remaining recycles when the redeal is bounded), used as the cycle-detection key and reusable elsewhere.
- `solver-engine`: The depth-first brute-force search — legal-move generation with no-op (and optional equivalence) pruning, per-path cycle detection, budget enforcement, stop-at-first-win, and the result/statistics (winnable, example moveset, nodes, time, peak logical memory).
- `solver-cli`: The `--solve` CLI mode and its flags and output formatting.

### Modified Capabilities

<!-- None. The core specs (klondike-domain-model, -rules-engine, -scoring) and the CLI specs are consumed unchanged; the solver reads the model through its existing public API. -->

## Impact

- **New library module**: `src/model/../solver` (or `src/solver`), std-only, keeping the library dependency-free.
- **Consumes existing public API**: `GameState` (Clone/Eq), `legal_moves`, `apply`, `is_won`, and pile/`config`/`recycles_done` accessors. No model changes required; the encoding is derived from public accessors.
- **CLI**: `main.rs`/args gain a `--solve` mode and solver budget flags (reusing clap); the interactive game is unaffected when `--solve` is absent.
- **Performance caveat (intended)**: per-path DFS without global memoization re-explores states reached by different paths, so it is slow and may be *inconclusive within budget* on many full deals. This is accepted for the baseline; the encoding and result types are designed so a future transposition-table solver can reuse them.
