## ADDED Requirements

### Requirement: Card representation
The system SHALL represent a playing card as a value composed of exactly one `Rank` (Ace, Two … Ten, Jack, Queen, King) and one `Suit` (Clubs, Diamonds, Hearts, Spades). Each card SHALL expose its `Color` (Hearts and Diamonds are red; Clubs and Spades are black), derived from its suit. A card SHALL also carry a face-orientation flag indicating whether it is face-up or face-down.

#### Scenario: Deriving card color from suit
- **WHEN** a card's suit is Hearts or Diamonds
- **THEN** its color is Red
- **WHEN** a card's suit is Clubs or Spades
- **THEN** its color is Black

#### Scenario: Rank ordering
- **WHEN** ranks are compared for sequencing
- **THEN** Ace is the lowest (rank value 1) and King is the highest (rank value 13), and every rank has a distinct value in that range

### Requirement: Standard 52-card deck
The system SHALL be able to construct a standard deck of exactly 52 unique cards — one card for each of the 13 ranks in each of the 4 suits, with no duplicates and no jokers.

#### Scenario: Constructing a fresh deck
- **WHEN** a new standard deck is constructed
- **THEN** it contains exactly 52 cards
- **AND** every (rank, suit) combination appears exactly once

### Requirement: Game piles
The system SHALL model the Klondike layout using standard terminology: a `Stock` (the face-down draw pile), a `Waste` (also called the talon, the face-up discard pile), four `Foundation` piles (one built up by suit from Ace to King), and seven `Tableau` columns (built down in alternating colors). Each pile SHALL preserve card order, and tableau columns SHALL distinguish their face-down and face-up portions.

#### Scenario: Foundation targets one suit
- **WHEN** a foundation pile holds one or more cards
- **THEN** every card in that foundation shares the same suit and forms an ascending Ace-upward sequence

#### Scenario: Tableau column exposes only its top face-up cards
- **WHEN** a tableau column contains face-down and face-up cards
- **THEN** the face-down cards occupy the bottom of the column and the face-up cards occupy the top, contiguously

### Requirement: Game configuration
The system SHALL expose a `GameConfig` that selects the draw mode (draw one card or draw three cards per stock draw) and the stock recycle policy (`redeal_limit` as an optional non-negative count, where absence means unlimited recycles). The configuration SHALL default to the classic Windows behavior: draw-three with unlimited recycles.

#### Scenario: Selecting draw-one
- **WHEN** a game is configured with draw mode "one"
- **THEN** each stock draw moves exactly one card to the waste (or fewer only when the stock has fewer than one remaining)

#### Scenario: Selecting draw-three
- **WHEN** a game is configured with draw mode "three"
- **THEN** each stock draw moves up to three cards to the waste, preserving their order

### Requirement: Deterministic seeded deal
The system SHALL create a new game from an unsigned integer seed such that the same seed and the same `GameConfig` always produce the identical initial deal. The shuffle SHALL use a documented, in-crate pseudo-random number generator (no reliance on system entropy or external RNG crates). The deal SHALL lay out the seven tableau columns with 1, 2, 3, 4, 5, 6, and 7 cards respectively, the top card of each column face-up and the rest face-down, with the remaining 24 cards forming the face-down stock and an empty waste and empty foundations.

#### Scenario: Reproducible deal from a seed
- **WHEN** two games are created with the same seed and the same configuration
- **THEN** their initial tableau, stock order, waste, and foundations are identical card-for-card

#### Scenario: Correct initial layout
- **WHEN** a new game is dealt
- **THEN** tableau column i (for i in 1..=7) contains exactly i cards with only its topmost card face-up
- **AND** the stock contains the remaining 24 cards face-down
- **AND** the waste and all four foundations are empty

### Requirement: Game state aggregate
The system SHALL provide a `GameState` aggregate that owns the stock, waste, four foundations, seven tableau columns, the originating seed, the active `GameConfig`, the current score, the elapsed-time reference, and the count of stock recycles performed. `GameState` SHALL be the single source of truth for a game in progress.

#### Scenario: State retains its seed and config
- **WHEN** a `GameState` is created from a seed and configuration
- **THEN** the state reports that same seed and configuration for the life of the game
