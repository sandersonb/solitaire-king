## Context

The `klondike` library exposes a complete rules engine (`legal_moves`, `GameState::apply`, `is_won`) over a `Clone`/`Eq` `GameState`, plus pile/config/`recycles_done` accessors. This change adds the first automatic solver: a naive brute-force DFS whose purpose is a *correct, measurable baseline*, not speed. It must (a) decide winnability within a budget, (b) return the first winning line found, and (c) report time and logical memory — and along the way it pins down three reusable concepts: a compact position encoding, no-op moves, and equivalent moves.

## Goals / Non-Goals

**Goals:**
- A correct DFS that finds a win when one is reachable within budget, returning the first winning line.
- A compact, canonical position encoding used for per-path cycle detection (and reusable later for save/replay and transposition tables).
- A **provably-safe** no-op reduction, and an **off-by-default** equivalence reduction with an empirical differential validator.
- Time + logical-memory reporting; a `--solve` CLI mode.
- Keep the library std-only.

**Non-Goals:**
- Any performance solver — no transposition table / global memoization, no heuristics, no parallelism (explicitly future work).
- Minimal/optimal solutions; proving unwinnability (budget exhaustion is "inconclusive").
- Save/replay file formats.

## Decisions

### Per-path cycle detection, no global memoization (the naive baseline)
Cycle detection uses only the positions on the current DFS path (a `HashSet` of encodings, pushed on descent and popped on backtrack), purely to prevent infinite loops within a descent. This change deliberately does **not** use a global transposition table — that is the optimization reserved for the next solver, and keeping it out is what makes this the honest brute-force baseline. Consequence: positions reached by different branches are re-explored, so the search is genuinely exponential and may be **inconclusive within budget** on many full deals. Accepted for the baseline; the encoding and `SolveResult` are shaped so a later transposition-table solver can drop in and be measured against this one. The search returns at the first winning line.

### Make/clone-per-child (naive), backtrack the path set
For each candidate move we `clone()` the `GameState`, `apply` the move, and recurse; on return we pop the path-set entry. No make/unmake yet (the model has no undo). Rationale: simplest correct approach; `GameState` is small. Memory stays low because clones are dropped on backtrack — logical memory is dominated by the path set (depth) and retained winning movesets, not by a frontier. A make/unmake optimization is deferred.

### Position encoding
Encode a position as a `Vec<u8>` (wrapped in a `PositionKey` newtype implementing `Hash`/`Eq`):
- **Cards**: one byte per card — rank (1–13) in the low nibble, suit (0–3) in two bits, face-up in one bit.
- **Tableau**: each of the 7 columns as `[len, card…]`, in column order (column order is preserved — collapsing empty-column permutations is a *future* canonicalization, deliberately not done here to keep the encoding provably exact).
- **Stock / waste**: `[len, card…]` in order.
- **Foundations**: canonical by suit — 4 bytes, the top rank present for each suit in fixed suit order (Clubs, Diamonds, Hearts, Spades), `0` if empty. This makes foundation-slot permutations encode identically (safe: a foundation is fully determined by suit+top).
- **Redeal**: append recycles-remaining only when the redeal limit is bounded (it changes legality); omit under unlimited.

We store full bytes (not a lossy hash) in the path set so cycle detection cannot false-positive and prune a real winning line. Path sets are shallow (a winning line is at most a few hundred moves), so this is cheap.

### No-op classification (provably safe)
Skip a `TableauToTableau { from, to, count }` move iff **all** hold:
1. it reveals no face-down card (the card beneath the moved run is face-up, or `from` becomes empty);
2. it lands on an interchangeable host — either (empty→empty, i.e. a King-headed run moved between empty columns) or the run's bottom card moves from a card `A` onto a card `C` with the **same rank and same color**;
3. the card exposed at `from` after the move (its new top, if any) has **no legal move** in the resulting position.
This is a symmetry/futility reduction: the successor is isomorphic (by swapping interchangeable slots/hosts) to a position reachable without the move and reveals nothing, so any win through it is reachable without it. Draws, recycles, and any foundation-touching or card-revealing move are excluded by construction. **Soundness is guarded empirically** by the differential validator (below), run across many seeds in tests.

### Equivalence reduction: off by default, plus a differential validator
"Equivalent moves" (a freshly available card with two interchangeable destinations) is implemented behind an off-by-default flag because, unlike the no-op rule, its safety is not obviously provable ("revealed cards can always be moved around" needs evidence). The **differential validator** runs the search twice — with and without a chosen optional rule — over a set of seeds under the same budget and asserts the solvable/unsolvable verdict is identical; any divergence is reported. This is the "scientific evaluation" mechanism and doubles as a regression guard for the no-op rule too.

### Budget and result
`SolveBudget { max_nodes, max_time }` (optional, with defaults `max_nodes = 10_000_000`, `max_time = 15s`). The search checks the budget at each node, stops on either limit, and returns immediately at the first win. `SolveResult { solvable, moveset: Option<Vec<Move>>, nodes_expanded, max_depth, elapsed, peak_logical_bytes, peak_positions, budget_exhausted }`. `solvable == false && budget_exhausted == true` ⇒ **inconclusive**, never "unwinnable".

### Logical memory accounting
Peak logical bytes = max over the search of `path_set_positions * encoded_state_bytes + retained_winning_moveset_bytes + working_state_bytes`. Portable, deterministic, no OS calls. We also report peak positions held. (OS RSS was considered and rejected for this change to avoid a libc dependency and non-determinism.)

### Module layout and CLI
New `src/solver/` module: `encode.rs` (PositionKey), `classify.rs` (no-op / equivalence predicates), `search.rs` (DFS, budget, result), `mod.rs` (public `solve(seed, config, budget, options) -> SolveResult`). CLI `main.rs` gains a `--solve` branch that builds the config, runs `solve`, and prints the report; solver budget flags are added to the clap args. The interactive path is untouched when `--solve` is absent.

## Risks / Trade-offs

- **Exponential blow-up / inconclusive results on real deals** → Budgets bound runtime; output distinguishes "inconclusive" from "unwinnable"; docs set expectations that this is a baseline, with transposition tables coming later.
- **No-op rule accidentally unsound (prunes a needed move)** → Conservative sufficient condition + differential validator across many seeds in the test suite; the rule only ever skips symmetric/futile tableau shuffles.
- **Clone-per-node cost** → Accepted for the baseline; make/unmake is a clean future optimization that does not change the public result types.
- **Encoding not collapsing empty-column symmetry** → Chosen for exactness/safety now; a canonicalizing encoding is a future optimization measurable against this baseline.
- **Which winning line is returned depends on move-generation order** → The first-win result need not be minimal or canonical; it is whatever the move ordering reaches first. Documented; acceptable for the baseline.

## Open Questions

- Default budgets — **resolved (confirmed)**: `max_nodes = 10,000,000` and `max_time = 15s`, both flag-tunable. `max_paths` was removed because the solver stops at the first winning line.
- Whether `--solve` should also accept an explicit position (not just a seed) later; out of scope now (seed-only).
