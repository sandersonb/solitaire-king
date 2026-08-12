# klondike-scoring Specification

## Purpose

Defines Microsoft Windows Solitaire "Standard" scoring for Klondike: the per-move point events, the stock recycle penalty, and the optional timed bonus and time penalty, all driven by move application.

## Requirements

### Requirement: Standard scoring point events
The system SHALL maintain a running score using Microsoft Windows Solitaire "Standard" scoring, updating it as moves are applied. The point events SHALL be: waste-to-tableau +5, turn over a tableau card +5, waste-to-foundation +10, tableau-to-foundation +10, and foundation-to-tableau −15. The score SHALL never drop below zero; any event that would take it negative SHALL clamp the score to zero. The exact point values SHALL be defined as named constants so they are auditable and adjustable.

#### Scenario: Moving a tableau card to a foundation scores ten
- **WHEN** a tableau card is moved onto a foundation
- **THEN** the score increases by 10

#### Scenario: Flipping a tableau card scores five
- **WHEN** applying a move auto-flips a newly exposed tableau card face-up
- **THEN** the score increases by 5

#### Scenario: Returning a foundation card penalizes fifteen
- **WHEN** a card is moved from a foundation back to a tableau column
- **THEN** the score decreases by 15, clamped so it is never below zero

### Requirement: Stock recycle penalty
The system SHALL apply the Windows recycle penalty when the waste is recycled into the stock. In draw-three mode there SHALL be no recycle penalty. In draw-one mode, each recycle after the first pass through the deck SHALL deduct 100 points, clamped so the score is never below zero.

#### Scenario: No penalty recycling in draw-three
- **WHEN** the waste is recycled while in draw-three mode
- **THEN** the score is unchanged by the recycle

#### Scenario: Draw-one recycle deducts one hundred
- **WHEN** the waste is recycled in draw-one mode after the first pass
- **THEN** the score decreases by 100, clamped so it is never below zero

### Requirement: Timed scoring
The system SHALL support an optional timed mode. When timed mode is enabled, the system SHALL deduct 2 points for every 10 seconds of elapsed play, and upon winning SHALL add a time bonus computed from elapsed seconds (larger bonus for faster completion). When timed mode is disabled, elapsed time SHALL NOT affect the score. Timed mode SHALL be selectable independently of the draw and redeal settings.

#### Scenario: Time penalty accrues while timed
- **WHEN** timed mode is enabled and 20 seconds of play have elapsed
- **THEN** the elapsed-time penalty applied to the score is 4 points

#### Scenario: No time effect when untimed
- **WHEN** timed mode is disabled
- **THEN** elapsed play time never changes the score

#### Scenario: Win bonus in timed mode
- **WHEN** timed mode is enabled and the game is won
- **THEN** a positive time bonus derived from elapsed seconds is added to the final score
