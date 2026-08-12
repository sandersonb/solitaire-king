## Context

This is a greenfield Rust project. The end goal is a Klondike Solitaire system with a CLI, an interactive UI, and an automatic solver. This change delivers only the core model: pure, deterministic, I/O-free domain types plus a classic-rules engine and Windows Standard scoring. Every later surface (CLI, UI, solver) will depend on this core, so the priorities are correctness, determinism (reproducible from a seed), and a clean public API. Terminology follows the standard Klondike vocabulary (stock, waste/talon, foundation, tableau) per the Wikipedia reference.

## Goals / Non-Goals

**Goals:**
- A complete set of domain types for classic Klondike using standard terminology.
- Deterministic deal: `(seed, config)` fully determines the initial layout, using an in-crate PRNG (no external RNG crate, no system entropy).
- A rules engine that enumerates legal moves, applies moves (with auto-flip and stock recycle), enforces the redeal limit, and detects a win.
- Configurable draw mode (1 or 3) and recycle policy (`redeal_limit: Option<u32>`).
- Windows Standard scoring including the optional timed bonus/penalty, driven by move application.
- Thorough unit tests as the primary verification surface (no binary yet).

**Non-Goals:**
- No CLI, TUI, or GUI (later changes).
- No automatic solver (later change) — but the API is shaped so a solver can drive it (enumerate → apply).
- No game variants beyond classic; no Vegas/gambling scoring.
- No serialization/persistence format, networking, or async.

## Decisions

### Crate shape: library-only, module tree under `model`
A single library crate (`src/lib.rs`) with a `model` module split into `card`, `deck`, `pile`, `deal`, `state`, `moves`, `rules`, `score`, and `rng`. No `main.rs` in this change. Rationale: the core must be consumable by three different front-ends; keeping it a library with no I/O keeps it testable and reusable. Alternative considered: put everything in `lib.rs` — rejected for maintainability.

### Card modeling: enums for `Suit`/`Rank`, small `Copy` `Card` struct
`Suit` and `Rank` are `#[repr(u8)]` C-like enums; `Rank` values run 1..=13 (Ace=1, King=13) to make sequencing arithmetic trivial. `Color` is derived from `Suit`. `Card` is a `Copy` struct `{ rank, suit, face_up }`. Rationale: cards are tiny value types; `Copy` keeps pile manipulation ergonomic and cheap. Alternative: encode a card as a single `u8` (0..52) — more compact but less readable; deferred as a possible later optimization since the public type can stay stable.

### Piles: `Vec<Card>` with defined top-of-pile end
Each pile wraps a `Vec<Card>`. Convention: the **last** element is the top (the actionable card). Tableau columns track their face-up run by the `face_up` flag on cards (face-down cards are always a contiguous bottom prefix). Rationale: `Vec` push/pop matches stock/waste/foundation semantics directly; a single ordering convention avoids off-by-one confusion. Alternative: `VecDeque` — unnecessary since all real operations happen at one end.

### Deterministic RNG: in-crate SplitMix64 → Fisher–Yates
Implement a tiny, well-known 64-bit PRNG (SplitMix64) seeded by the game seed, and shuffle the deck with an unbiased Fisher–Yates using it. Rationale: no external dependency, trivially auditable, portable across platforms, and fully reproducible — essential for replay/sharing and for deterministic solver/test runs. Alternative: the `rand` crate with a seeded `StdRng` — rejected to avoid a dependency and because `StdRng`'s algorithm/output is not guaranteed stable across versions.

### Move application returns a Result and reports scored events
`apply_move(&mut GameState, Move) -> Result<(), MoveError>`; illegal moves return `Err` and leave the state untouched (validated by re-checking legality before mutating). Move application internally emits the scoring events it caused so the `score` module can update the running total in the same step. Rationale: keeps legality, state transition, and scoring consistent and centralized. Alternative: pure `apply(state) -> state` returning a new state — cleaner functional style, but in-place mutation with rollback-by-validation is simpler and cheaper for the interactive/solver loops; can revisit if we later need cheap state snapshots.

### Scoring as named constants + a `ScoreConfig`
Point values live as named constants and the timed-mode toggle lives in a `ScoreConfig` (or a flag on `GameConfig`). Windows Standard values used: waste→tableau +5, flip tableau card +5, waste→foundation +10, tableau→foundation +10, foundation→tableau −15; draw-one recycle −100 per pass after the first (no penalty in draw-three); timed: −2 per 10s elapsed, plus a win-time bonus. Score is clamped at 0. Rationale: named constants make the exact rules auditable and easy to adjust if we refine them. The timed bonus formula is treated as a single documented function so it can be tuned. Open for confirmation (see Open Questions).

### Time handling: injectable elapsed-seconds, not wall clock
The core does not read the system clock. Timed scoring is computed from an elapsed-seconds value supplied by the caller (the front-end owns the clock). Rationale: preserves determinism and testability of the pure core. Alternative: read `Instant::now()` inside the model — rejected because it makes the model non-deterministic and hard to test.

## Risks / Trade-offs

- **Exact Windows scoring values may differ from a specific Windows edition** → Values are centralized as named constants with citations; easy to correct in one place. Flagged in Open Questions for user confirmation before implementation.
- **In-place mutation makes state snapshots non-trivial for the future solver** → The `Move` set is small and moves are cheaply invertible; if the solver needs backtracking, we can add `#[derive(Clone)]` on `GameState` (all fields are `Clone`) or implement undo later without changing the public shape.
- **Encoding tableau face-up state via a per-card flag** risks an invariant violation (a face-down card above a face-up one) → A single `debug_assert`-backed invariant check on tableau columns, exercised in tests, guards this.
- **Reproducibility depends on never changing the shuffle algorithm** → SplitMix64 + Fisher–Yates is pinned and documented; any future change to it is a breaking change to deal reproducibility and will be treated as such.

## Open Questions

- Confirm the precise Windows Standard scoring numbers above (especially waste→tableau = +5 and the draw-one recycle −100). If you have a specific Windows edition to match, I'll pin those exact values.
- Timed win-bonus formula: adopt the commonly cited `bonus = 700000 / elapsed_seconds` (for elapsed_seconds above a small floor), or a simpler tunable formula? Defaulting to the documented `700000 / seconds` unless you prefer otherwise.
- Should `GameState` derive `Clone` now (to ease the future solver and undo) even though this change doesn't use it? Leaning yes, since it's free and forward-compatible.
