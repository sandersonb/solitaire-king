## MODIFIED Requirements

### Requirement: On-screen controls
The GUI SHALL provide on-screen buttons for the core session actions so the game is
fully playable by touch alone: a combined **Undo/Redo** button where a tap undoes the
last move and a press-and-hold redoes one move; a **New game** button; and a
**Settings** button. The solvability indicator SHALL also act as a button. Activating
any control (click or tap) SHALL perform its action; the controls SHALL be present on
touch / mobile layouts and MAY be shown on desktop.

#### Scenario: Undo button works by touch
- **WHEN** the player taps the on-screen Undo button after a move
- **THEN** the last move is undone, with no keyboard required

#### Scenario: Hold redoes
- **WHEN** the player presses and holds the Undo button after undoing a move
- **THEN** one undone move is redone

#### Scenario: New game button works by touch
- **WHEN** the player taps the on-screen New button
- **THEN** a fresh game is dealt

#### Scenario: Settings button opens settings
- **WHEN** the player taps the Settings button
- **THEN** the settings dialog opens

## ADDED Requirements

### Requirement: Interactive solver actions
Activating the solvability indicator SHALL open the state-dependent solver overlay
(except while a check runs). When a solution is known, the overlay's Auto-solve
action and the **Shift+A** shortcut SHALL start auto-solving the current deal; Shift+A
SHALL do nothing when no solution is known. Overlays SHALL be dismissible (a close
control or clicking outside), and while an overlay or dialog is open it SHALL take
input priority over the board.

#### Scenario: Indicator opens the overlay
- **WHEN** the player activates the indicator while no check is running
- **THEN** the solver overlay for the current status opens

#### Scenario: Auto-solve from the overlay
- **WHEN** the player activates Auto-solve in the solvable overlay
- **THEN** auto-solving begins

#### Scenario: Shift+A auto-solves when a solution is known
- **WHEN** the player presses Shift+A and a solution is known for the current position
- **THEN** auto-solving begins

#### Scenario: Overlay takes input priority
- **WHEN** an overlay or dialog is open and the player clicks
- **THEN** the click is handled by the overlay, not the board
