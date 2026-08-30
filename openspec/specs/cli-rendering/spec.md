# cli-rendering Specification

## Purpose

Defines how the CLI presents the Klondike game state to the terminal: board rendering of all piles, color and Unicode presentation, the status line, pile alignment under key headers, selection feedback, and help/message feedback.

## Requirements

### Requirement: Board rendering
The CLI SHALL render the full game state to the terminal each cycle: the seven tableau columns, the stock, the waste, and the four foundations. Face-up cards SHALL show their rank and suit; face-down cards SHALL be shown as a distinct concealed marker; empty piles SHALL be shown as a distinct empty marker.

#### Scenario: Tableau reflects face-up and face-down cards
- **WHEN** a tableau column has face-down cards beneath a face-up run
- **THEN** the face-down cards render as concealed markers and the face-up cards render with rank and suit, in order

#### Scenario: Stock and waste render
- **WHEN** the stock has cards and the waste has cards
- **THEN** the stock renders as a concealed pile (or a count) and the waste renders showing its top card (per draw mode, up to the last few)

#### Scenario: Empty foundation renders distinctly
- **WHEN** a foundation is empty
- **THEN** it renders as an empty marker distinguishable from a face-down card

### Requirement: Color and Unicode presentation
The CLI SHALL use UTF-8 suit symbols (♠ ♥ ♦ ♣) and ANSI color to aid readability: red suits (hearts, diamonds) SHALL render in a red color and black suits (clubs, spades) in a contrasting color. Face-down cards SHALL be visually dimmed or distinct from face-up cards.

#### Scenario: Suit colors
- **WHEN** a red-suit card and a black-suit card are displayed
- **THEN** the red-suit card is shown in red and the black-suit card in a contrasting color

#### Scenario: Face-down distinct from face-up
- **WHEN** face-down and face-up cards are displayed
- **THEN** the face-down cards are visually distinct (dimmed or a concealed glyph)

### Requirement: Status line
The CLI SHALL display a status area showing the current seed (as the pronounceable seed string), the number of moves made, the current score, and the elapsed play time.

#### Scenario: Status shows game metrics
- **WHEN** the board is rendered
- **THEN** the seed (as the pronounceable seed string), move count, score, and elapsed time are all visible

### Requirement: Pile alignment
Each pile's cell SHALL render in the same terminal column as its key header, so that the stock, waste, and the four foundations line up exactly beneath their `[8] [9] [0] [-]` labels and the tableau columns line up beneath `[1]`–`[7]`, regardless of how many waste cards are shown.

#### Scenario: Foundation cells sit under their headers
- **WHEN** the board is rendered with any number of cards in the waste
- **THEN** each foundation cell appears in the same column as its `[8]`/`[9]`/`[0]`/`[-]` header

### Requirement: Selection feedback
When the player has chosen a source but not yet a target, the CLI SHALL visually indicate the pending source (for example by highlighting it) and prompt for the next key. This SHALL include the stock/waste when it is the pending source, including an empty stock (shown as `( )`), so the player can see that a further `space` will recycle.

#### Scenario: Pending source highlighted
- **WHEN** the player presses a source key and a target has not yet been chosen
- **THEN** the selected source pile is highlighted and the status area prompts for a target

#### Scenario: Empty stock highlights when selected
- **WHEN** the stock is empty and the player presses `space` while the waste has cards
- **THEN** the empty stock marker `( )` is highlighted, indicating a further `space` will recycle

### Requirement: Help and message feedback
The CLI SHALL provide a help/legend overlay toggled with `?` that explains the keys, and SHALL show a transient message when an attempted move is rejected or an action is not possible.

#### Scenario: Help overlay lists keys
- **WHEN** the player presses `?`
- **THEN** an overlay appears listing the key bindings, and pressing `?` again (or a dismiss key) hides it

#### Scenario: Rejected move message
- **WHEN** the player attempts an illegal move
- **THEN** a brief message indicates the move was not allowed and the board state is unchanged
