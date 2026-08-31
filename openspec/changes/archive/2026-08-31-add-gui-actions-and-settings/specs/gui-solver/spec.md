## MODIFIED Requirements

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

## ADDED Requirements

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
