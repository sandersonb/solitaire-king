# gui-solver Specification

## Purpose

The GUI's background solvability assist: it runs the solver on the current
position without blocking play, shows a live indicator of whether the deal is
still winnable, and warns the player (once) when the position becomes provably
unwinnable — while reusing solver work across checks and staying correct across
undo/redo.

## Requirements

### Requirement: Background solvability checks
The GUI SHALL evaluate the current position with the solver in the background:
once shortly after a new deal, and again whenever play has been idle (no applied
move or pointer interaction) for about three seconds. Each check SHALL be bounded
to roughly one second of solver work. A check MAY end **inconclusive**, and that
is a normal, expected outcome.

#### Scenario: Check after a new deal
- **WHEN** a new game is dealt
- **THEN** the GUI starts a background solvability check of the opening position

#### Scenario: Check after an idle pause
- **WHEN** the player has made no move or interaction for about three seconds and the current position's solvability is not already known
- **THEN** the GUI starts a background solvability check of the current position

#### Scenario: A check is time-bounded
- **WHEN** a background check has run for about one second without a decisive result
- **THEN** the check stops and the position is reported as inconclusive

### Requirement: Non-blocking cooperative execution
The background solve SHALL run cooperatively in small slices spread across frames
so the game never freezes, and SHALL behave the same on native and WebAssembly
(the deployed build is single-threaded). The interactive frame rate SHALL be
maintained while a check runs.

#### Scenario: Play continues during a check
- **WHEN** a background check is in progress
- **THEN** the board still renders and responds to input every frame

#### Scenario: Interaction preempts a check
- **WHEN** the player makes a move while a check is running
- **THEN** the check for the old position is abandoned and does not change the new position's status

### Requirement: Persistent search knowledge
The GUI SHALL preserve the solver's accumulated search knowledge (its
transposition table) in memory across successive checks and across moves within a
game, so later checks reuse prior work. Re-evaluating a position that was already
explored SHALL be fast.

#### Scenario: Later checks reuse prior work
- **WHEN** a second check runs after earlier checks in the same game
- **THEN** it reuses the retained table rather than starting from no knowledge

#### Scenario: Re-reaching an explored position is fast
- **WHEN** the current position was already proven unwinnable earlier in the game
- **THEN** its status is recognized from the retained knowledge without a new full search

### Requirement: Solvability indicator
The GUI SHALL display an on-screen indicator of the current position's solvability status with four distinct visuals: a solution exists, proven unwinnable, a check is currently running, and uncertain/inconclusive (including not-yet-checked). The indicator SHALL be an interactive button: activating it (click or tap) SHALL open a small state-dependent overlay describing the status and offering the relevant action, except while a check is running (Working), which SHALL have no action.

#### Scenario: Running indicator during a check
- **WHEN** a background check is in progress
- **THEN** the indicator shows the running (in-progress) visual

#### Scenario: Decisive result updates the indicator
- **WHEN** a check finishes proving the position solvable or unwinnable
- **THEN** the indicator shows the corresponding solvable or unwinnable visual

#### Scenario: Inconclusive shows uncertain
- **WHEN** a check ends inconclusive
- **THEN** the indicator shows the uncertain visual

#### Scenario: Activating the indicator opens the status overlay
- **WHEN** the player activates the indicator and a check is not running
- **THEN** a small overlay appears describing the current solvability status and any available action

#### Scenario: Working indicator is inert
- **WHEN** the player activates the indicator while a check is running
- **THEN** nothing happens

### Requirement: Unwinnable dialog
When the current position is proven unwinnable, the GUI SHALL present a dialog
stating the deal cannot be won and offering to continue playing or deal a new
game. The dialog SHALL NOT nag: once the player dismisses it (by continuing), it
SHALL NOT reappear until solvability returns (e.g., after an undo to a winnable
position).

#### Scenario: Dialog on proven unwinnable
- **WHEN** a check proves the current position unwinnable and the player has not already dismissed the dialog for this unwinnable streak
- **THEN** a dialog appears offering Continue and New game

#### Scenario: Continue keeps playing without nagging
- **WHEN** the player chooses Continue on the unwinnable dialog
- **THEN** the dialog closes and does not reappear while the game remains unwinnable

#### Scenario: New game from the dialog
- **WHEN** the player chooses New game on the unwinnable dialog
- **THEN** a fresh deal starts and a new solvability check begins

### Requirement: Per-position, undo-aware status
Solvability status SHALL apply to the current position, not the whole game. A
proven-unwinnable result SHALL NOT trigger a fresh search again for that same
position, but any change to the current position (move, undo, redo, or new game)
SHALL re-establish the status for the new position — clearing an unwinnable state
when undo returns to a winnable or not-yet-decided position.

#### Scenario: Undo can restore a winnable state
- **WHEN** the position is unwinnable and the player undoes back to a position that is (or may be) winnable
- **THEN** the unwinnable state is cleared and the position is (re-)checked

#### Scenario: Known-unwinnable position is not re-searched
- **WHEN** the current position is already proven unwinnable
- **THEN** the GUI does not start another search for it

#### Scenario: Forward play from unwinnable stays unwinnable without a dialog
- **WHEN** the player continued from an unwinnable position and makes further moves
- **THEN** the successor positions are recognized as unwinnable and no new dialog is shown

### Requirement: Solver action overlay
Activating the indicator SHALL open an overlay whose content depends on the status:
solvable shows that a solution exists in a stated number of moves and offers an
**Auto-solve** action; unwinnable states the deal cannot be won but that undoing may
reopen a solution; uncertain suggests that making more moves may determine
solvability. The overlay SHALL be dismissible.

#### Scenario: Solvable overlay offers auto-solve
- **WHEN** the overlay opens for a solvable position
- **THEN** it states a solution exists in the found number of moves and offers an Auto-solve action

#### Scenario: Unwinnable overlay explains undo
- **WHEN** the overlay opens for an unwinnable position
- **THEN** it states the deal cannot be won and that undoing may reopen a solution

#### Scenario: Uncertain overlay suggests more play
- **WHEN** the overlay opens for an uncertain position
- **THEN** it suggests that making more moves may reveal whether the deal is solvable

### Requirement: Retain the winning line
When a check proves the current position solvable, the assist SHALL retain the
winning move-sequence it found and its length, so the overlay can report the number
of moves and auto-solve can replay it. The retained line SHALL correspond to the
current position.

#### Scenario: Move count reflects the found line
- **WHEN** a position is proven solvable
- **THEN** the overlay reports the number of moves in the retained winning line

### Requirement: Auto-solve
When the current position is solvable, the GUI SHALL offer to auto-solve it — via
the overlay's Auto-solve action or the **Shift+A** shortcut — playing the retained
winning line to completion. Auto-solve SHALL be available only when a solution
exists; Shift+A SHALL do nothing otherwise.

#### Scenario: Auto-solve plays the solution
- **WHEN** the player triggers auto-solve on a solvable position
- **THEN** the retained winning line is played to the won state

#### Scenario: Shift+A requires a known solution
- **WHEN** the player presses Shift+A while no solution is known for the current position
- **THEN** nothing happens

### Requirement: Background solver can be disabled
The GUI SHALL honor a setting that enables or disables the background solver. When
disabled, no checks SHALL run and the indicator SHALL reflect that solvability is
not being evaluated.

#### Scenario: Disabled solver runs no checks
- **WHEN** the background solver is disabled in settings
- **THEN** no solvability checks run and no unwinnable dialog appears

#### Scenario: Re-enabling resumes checking
- **WHEN** the background solver is re-enabled
- **THEN** the current position is evaluated again
