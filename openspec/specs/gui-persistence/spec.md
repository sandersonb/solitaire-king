# gui-persistence Specification

## Purpose

Local persistence for the GUI: storing small amounts of data that survive across
sessions — using the browser's storage on web and a local file on native — and the
records built on it, starting with the high-score record.

## Requirements

### Requirement: Cross-platform local persistence
The GUI SHALL persist small key-value data locally so it survives restarts. On the
web build it SHALL use the browser's local storage; on native it SHALL use a file
in the user's configuration directory (e.g. `~/.config/<app>/`). Reading absent or
unreadable/corrupt data SHALL be treated as "no data" and SHALL NOT crash or block
the game — persistence is best-effort and never on the critical path of play.

#### Scenario: Data survives a restart
- **WHEN** the game saves a value and is later relaunched (browser reload on web, or restart on native)
- **THEN** the saved value is read back on the next launch

#### Scenario: Missing or corrupt data is tolerated
- **WHEN** no stored data exists yet, or the stored data cannot be parsed
- **THEN** the game proceeds as if there were no data, without error

### Requirement: Persisted high-score record
The GUI SHALL keep a single best-result record consisting of the highest final
score achieved by play, the elapsed time of the game that achieved it, and a
timestamp of when it was set. The record SHALL be loaded at startup and SHALL be
updated and saved whenever a game won by play finishes with a final score higher
than the stored high score (or when no record exists yet). A win reached by
auto-solve SHALL NOT update the record. Only the single best record is kept.

#### Scenario: First win sets the record
- **WHEN** the first game is won by play and no record exists
- **THEN** the record is created with that game's score, time, and the current timestamp, and saved

#### Scenario: A higher score updates the record
- **WHEN** a game is won by play with a final score higher than the stored high score
- **THEN** the record is replaced with the new score, its time, and the current timestamp, and saved

#### Scenario: A lower score leaves the record unchanged
- **WHEN** a game is won by play with a final score at or below the stored high score
- **THEN** the stored record is unchanged

#### Scenario: Auto-solve does not affect the record
- **WHEN** a game is completed by auto-solve
- **THEN** the high-score record is not updated

#### Scenario: Record loads at startup
- **WHEN** the game launches and a record was previously saved
- **THEN** that record is available (e.g. to display) without replaying past games

### Requirement: Persisted play counters
The GUI SHALL keep two persisted lifetime counters: the number of games won by
play, and the number of new games started. The wins counter SHALL increment when a
game is won by play (a win reached by auto-solve SHALL NOT count). The new-games
counter SHALL increment each time a new game is dealt (including the initial deal
at launch and each subsequent new game). Both counters SHALL be loaded at startup,
incremented and saved as their events occur, and SHALL accumulate across sessions.

#### Scenario: Winning increments the wins counter
- **WHEN** a game is won by play
- **THEN** the persisted wins counter increases by one

#### Scenario: Auto-solve does not count as a win
- **WHEN** a game is completed by auto-solve
- **THEN** the wins counter is unchanged

#### Scenario: Starting a game increments the new-games counter
- **WHEN** a new game is dealt (at launch or via a new-game action)
- **THEN** the persisted new-games counter increases by one

#### Scenario: Counters accumulate across sessions
- **WHEN** the game is relaunched after some wins and new games
- **THEN** the counters continue from their previously saved totals

### Requirement: Persisted settings
The GUI SHALL persist the settings-dialog values — the draw mode, the deck choice
(Auto / Mobile / Standard), the background-solver toggle, and the show-seed toggle
— saving them when changed and restoring them on the next launch. Restored settings
SHALL apply as the session's starting values. (On native, an explicit launch
argument for a setting MAY take precedence over the persisted value for that
launch.)

#### Scenario: A changed setting persists
- **WHEN** the player changes a settings-dialog value and later relaunches the game
- **THEN** that value is restored as the starting value

#### Scenario: Defaults when nothing is saved
- **WHEN** the game launches with no persisted settings (or unreadable settings)
- **THEN** the default settings are used without error
