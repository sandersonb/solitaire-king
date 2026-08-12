## ADDED Requirements

### Requirement: Pile key mapping
The CLI SHALL map keys to piles consistently for both source and target selection: `1`–`7` select tableau columns 1–7, the keys `8` `9` `0` `-` select the four foundations, and `space` selects the stock/waste. The same mapping SHALL apply whether a key is chosen as a move's source or its target.

#### Scenario: Digit keys select tableau columns
- **WHEN** the player presses `3`
- **THEN** tableau column 3 is the addressed pile

#### Scenario: Foundation keys
- **WHEN** the player presses `8`, `9`, `0`, or `-`
- **THEN** the corresponding foundation is the addressed pile

### Requirement: Source-then-target move construction
The CLI SHALL build a move from two selections: a source pile, then a target pile. After a source is selected the CLI SHALL wait for a target; choosing a target SHALL attempt the corresponding move. `Esc` SHALL cancel a pending source selection.

#### Scenario: Two-key move
- **WHEN** the player presses a source key and then a target key that form a legal move
- **THEN** that move is applied

#### Scenario: Cancel pending source
- **WHEN** the player has selected a source and presses `Esc`
- **THEN** the pending selection is cleared and no move is made

### Requirement: Stock dealing and recycling
Pressing `space` SHALL engage the stock/waste. When there is no waste card available to move, `space` SHALL deal from the stock to the waste immediately (per the draw mode); when the stock is empty it SHALL recycle the waste back into the stock (subject to the redeal limit). When a waste card is available, `space` SHALL select the stock/waste as the move source, and a subsequent `space` SHALL perform the deal so an existing waste card is never dealt over unintentionally.

#### Scenario: Deal from stock
- **WHEN** the stock is non-empty and the player deals with `space`
- **THEN** cards are moved from the stock to the waste according to the draw mode

#### Scenario: Recycle when stock empty
- **WHEN** the stock is empty, the waste is non-empty, the redeal limit permits it, and the player deals
- **THEN** the waste is recycled back into the stock

#### Scenario: Recycle blocked at limit
- **WHEN** the stock is empty and the redeal limit has been reached
- **THEN** dealing does not recycle and a message indicates no more redeals are allowed

### Requirement: Waste as a move source
With the stock/waste selected as the source (`space`), pressing a target pile key SHALL move the waste's top card to that pile if the move is legal.

#### Scenario: Move waste to a tableau column
- **WHEN** the stock/waste is the source and the player presses a tableau column key that yields a legal placement
- **THEN** the waste's top card is moved to that column

#### Scenario: Move waste to a foundation
- **WHEN** the stock/waste is the source and the player presses a foundation key that yields a legal placement
- **THEN** the waste's top card is moved to the appropriate foundation

### Requirement: Auto-assign with Enter
Pressing `Enter` SHALL auto-assign to a card's best legal destination without the player naming a target. With no source selected, `Enter` SHALL act on the waste's top card; with a source selected, `Enter` SHALL act on that source. The choice SHALL be greedy, favoring the move that relocates the most cards first and descending to the least (so the largest legal tableau run is preferred); ties SHALL be broken by preferring a foundation. If no legal move exists, no move is made and a message is shown.

#### Scenario: Auto-assign the waste top
- **WHEN** no source is selected and the player presses `Enter` while the waste's top card has a legal destination
- **THEN** that card is moved, preferring a foundation over a tableau column

#### Scenario: Auto-assign favors the largest run
- **WHEN** a tableau source is auto-assigned and several run lengths are legal
- **THEN** the move relocating the most cards is chosen, trying fewer cards only if a larger run has no legal destination

#### Scenario: Auto-assign a selected source
- **WHEN** a tableau or foundation source is selected and the player presses `Enter`
- **THEN** that pile's top card is moved to its best legal destination

#### Scenario: Auto-assign with no legal destination
- **WHEN** the player presses `Enter` and the addressed card has no legal destination
- **THEN** no move is made and a brief message is shown

### Requirement: Foundation targeting is forgiving
When a foundation is chosen as a move's target, the CLI SHALL route the card to the foundation of the correct suit if such a placement is legal, regardless of which of the four foundation keys was pressed. The four foundation keys SHALL still individually identify foundations when a foundation is the source of a move.

#### Scenario: Any foundation key accepts a legal card
- **WHEN** a card can legally go to its suit's foundation and the player picks any foundation key as the target
- **THEN** the card is placed on the correct foundation

### Requirement: Tableau run length selection
When a tableau-to-tableau move could involve more than one card, the CLI SHALL move the uniquely determined legal run when only one length is legal, and SHALL prompt the player for the number of cards only when more than one run length is legal.

#### Scenario: Unique run moved automatically
- **WHEN** exactly one run length is legal for a chosen tableau-to-tableau move
- **THEN** that run is moved without prompting

#### Scenario: Ambiguous run prompts for count
- **WHEN** more than one run length is legal for the chosen source and target
- **THEN** the player is prompted for how many cards to move, and the chosen count is applied

### Requirement: Illegal moves are rejected
The CLI SHALL only apply moves that the rules engine reports as legal. An input that does not correspond to a legal move SHALL leave the game state unchanged and SHALL surface brief feedback.

#### Scenario: Illegal target is rejected
- **WHEN** the player selects a source and a target that do not form a legal move
- **THEN** no move is applied, the state is unchanged, and a brief message is shown
