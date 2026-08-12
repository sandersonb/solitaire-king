## 1. Project scaffolding

- [x] 1.1 Run `cargo init --lib` (or create `Cargo.toml` + `src/lib.rs`) for a library crate named `klondike` (edition 2021), no external runtime dependencies
- [x] 1.2 Add a `.gitignore` for Rust (`/target`), and declare the `model` module in `src/lib.rs`
- [x] 1.3 Create empty module files: `src/model/{mod.rs,rng.rs,card.rs,deck.rs,pile.rs,deal.rs,state.rs,moves.rs,rules.rs,score.rs}` and wire them in `model/mod.rs`

## 2. Cards and deck (klondike-domain-model)

- [x] 2.1 Implement `Suit`, `Rank` (values 1..=13, Ace=1..King=13), and `Color`, with `Suit::color()` deriving red/black
- [x] 2.2 Implement the `Card` `Copy` struct `{ rank, suit, face_up }` with constructors and face flip helpers
- [x] 2.3 Implement `Deck::standard()` producing all 52 unique cards; unit-test count == 52 and uniqueness of every (rank, suit)

## 3. Deterministic RNG and shuffle (klondike-domain-model)

- [x] 3.1 Implement in-crate SplitMix64 PRNG seeded by `u64` in `rng.rs`
- [x] 3.2 Implement unbiased Fisher–Yates shuffle over the deck using the PRNG
- [x] 3.3 Unit-test: same seed → identical shuffle order; different seeds → different order (with high probability)

## 4. Piles and game state (klondike-domain-model)

- [x] 4.1 Implement `Stock`, `Waste`, `Foundation`, and `TableauColumn` pile types over `Vec<Card>` with the last element as the top; add a tableau face-up-prefix invariant check
- [x] 4.2 Implement `DrawMode { One, Three }` and `GameConfig { draw_mode, redeal_limit: Option<u32>, timed: bool }` with a classic default (draw-three, unlimited, untimed)
- [x] 4.3 Implement `GameState` aggregate holding stock, waste, `[Foundation; 4]`, `[TableauColumn; 7]`, `seed`, `config`, `score`, elapsed-time reference, and `recycles_done`

## 5. Deal (klondike-domain-model)

- [x] 5.1 Implement `GameState::new_with_seed(seed, config)` that shuffles then deals columns of 1..=7 (top card face-up), the remaining 24 to the stock, empty waste/foundations
- [x] 5.2 Unit-test the initial layout (column sizes, single face-up per column, stock == 24, empty waste/foundations)
- [x] 5.3 Unit-test deal reproducibility: same `(seed, config)` → identical `GameState` card-for-card

## 6. Moves and rules engine (klondike-rules-engine)

- [x] 6.1 Define the `Move` enum covering draw, recycle, waste→foundation, waste→tableau, tableau→tableau (single card or run), tableau→foundation, foundation→tableau
- [x] 6.2 Implement tableau placement legality (King on empty; else one-lower, opposite-color) including validating a moved face-up run is itself a valid sequence
- [x] 6.3 Implement foundation placement legality (Ace on empty; else same suit, one higher)
- [x] 6.4 Implement stock draw (respecting draw mode) and recycle (respecting `redeal_limit`, incrementing `recycles_done`)
- [x] 6.5 Implement `legal_moves(&GameState) -> Vec<Move>` enumerating all currently valid moves
- [x] 6.6 Implement `apply_move(&mut GameState, Move) -> Result<(), MoveError>`: validate, mutate, and reject illegal moves without side effects
- [x] 6.7 Implement the auto-flip rule when a tableau move exposes a face-down top card
- [x] 6.8 Implement `is_won(&GameState)` (all four foundations Ace→King)
- [x] 6.9 Unit-test legality (valid/invalid tableau + foundation placements), recycle-limit enforcement, auto-flip, and illegal-move-is-a-no-op

## 7. Scoring (klondike-scoring)

- [x] 7.1 Define scoring constants (waste→tableau +5, flip +5, waste→foundation +10, tableau→foundation +10, foundation→tableau −15) and a `ScoreConfig`/timed flag
- [x] 7.2 Wire move application to emit scoring events and update the running score, clamped at 0
- [x] 7.3 Implement the draw-one recycle penalty (−100 per pass after the first; none in draw-three)
- [x] 7.4 Implement timed scoring: −2 per 10s from injected elapsed seconds, plus the documented win-time bonus
- [x] 7.5 Unit-test each scoring event, the zero clamp, the recycle penalty (both draw modes), and timed penalty/bonus

## 8. Public API and verification

- [x] 8.1 Re-export the public surface from `lib.rs` (domain types, `GameConfig`, `GameState::new_with_seed`, `legal_moves`, `apply_move`, `is_won`, score accessors)
- [x] 8.2 Add a small integration test that deals a fixed seed, plays a scripted sequence of legal moves, and asserts on resulting state and score
- [x] 8.3 Run `cargo test`, `cargo fmt --check`, and `cargo clippy` clean
