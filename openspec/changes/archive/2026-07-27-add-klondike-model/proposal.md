## Why

We are building a Klondike Solitaire application that will eventually be playable by a human via CLI/UI and solvable by an automatic solver. All of those surfaces need a single, correct, deterministic, I/O-free core model to build on. This change establishes that core: the game's data structures ("nouns"), the classic-rules engine (legal moves, move application, win detection), and Microsoft Windows Solitaire "Standard" scoring — with a reproducible seeded deal so any game can be replayed or shared.

## What Changes

- Initialize a new Rust project (Cargo library crate) as the home for the game core.
- Introduce the **domain model** — the Klondike nouns using standard terminology: `Card`, `Suit`, `Rank`, `Color`, the `Stock`, `Waste`, four `Foundation` piles, seven `Tableau` columns, and a `GameState` aggregate that owns them plus the seed, configuration, and score.
- Introduce a **deterministic deal**: a documented, portable seeded PRNG shuffles a standard 52-card deck and lays down the classic tableau (1..7 face-down cards with one face-up per column), the rest forming the stock.
- Introduce a **rules engine** for the classic (standard) ruleset: enumerate legal moves, apply a move to produce a new/updated state, auto-flip exposed tableau cards, recycle the stock, and detect a win.
- Introduce **draw and redeal configuration**: draw-1 vs draw-3, and a configurable stock recycle limit (`redeal_limit: Option<u32>`, `None` = unlimited) with pass-count tracking.
- Introduce **Windows Standard scoring** (untimed events plus the optional timed bonus/penalty), driven by move application.
- Provide unit tests covering deal determinism, move legality, scoring events, and win detection.

Non-goals (explicitly deferred to later changes): CLI, interactive UI/TUI, the automatic solver, game variants beyond classic rules, Vegas/gambling scoring, and persistence/serialization formats.

## Capabilities

### New Capabilities

- `klondike-domain-model`: The card/pile data structures, standard-deck construction, deterministic seeded deal/shuffle, game configuration (draw mode, redeal limit), and the `GameState` aggregate.
- `klondike-rules-engine`: Classic-rules move enumeration, move application (including auto-flip and stock recycle), redeal-limit enforcement, and win detection.
- `klondike-scoring`: Microsoft Windows "Standard" scoring — point events for moves plus the optional timed bonus and time penalty.

### Modified Capabilities

<!-- None: this is a greenfield project; no existing specs. -->

## Impact

- **New project scaffolding**: `Cargo.toml`, `src/lib.rs`, and a `model` module tree (no `main.rs` yet — CLI is a later change).
- **Dependencies**: standard library only for the core; the seeded PRNG is implemented in-crate (no external RNG dependency) to keep the deal portable and reproducible. A dev-dependency test helper may be added if needed.
- **Public API surface**: the crate exposes the domain types, `GameConfig`, `GameState::new_with_seed(...)`, legal-move enumeration, `apply_move`, and score/win accessors — the contract every later surface (CLI, UI, solver) will consume.
- **No I/O, no async, no global state** — the core is a pure, deterministic library.
