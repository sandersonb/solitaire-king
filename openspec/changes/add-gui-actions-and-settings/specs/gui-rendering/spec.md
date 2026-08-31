## MODIFIED Requirements

### Requirement: On-screen control bar rendering
The GUI SHALL render an on-screen control bar containing the touch-usable buttons — a combined Undo/Redo button, New game, and Settings — plus the solvability indicator button, positioned so they do not overlap the playable piles, and SHALL give visual feedback when a control is pressed. The status area SHALL NOT render a keyboard-command help line.

#### Scenario: Control bar is drawn and does not cover piles
- **WHEN** the board is rendered on a layout that shows the control bar
- **THEN** the Undo/Redo, New, and Settings buttons and the solvability indicator are visible and do not overlap the stock, waste, foundations, or tableau

#### Scenario: No command help line
- **WHEN** the board is rendered
- **THEN** no on-screen line advertising the keyboard/drag commands is shown

## ADDED Requirements

### Requirement: Overlays, settings dialog, and loading rendering
The GUI SHALL render, on top of the board: the state-dependent solver overlay (its
text and any action button per status); the settings dialog (draw mode, solver
enable, seed visibility); and an asset-loading progress screen (a spinner or progress
bar) shown while assets load. Open overlays/dialogs SHALL dim or otherwise separate
themselves from the board.

#### Scenario: Solver overlay is drawn for the status
- **WHEN** the solver overlay is open
- **THEN** it renders the message and any action appropriate to the current solvability status

#### Scenario: Settings dialog is drawn
- **WHEN** the settings dialog is open
- **THEN** it renders the draw-mode choice, the solver-enable toggle, and the seed-visibility toggle

#### Scenario: Loading progress is drawn
- **WHEN** assets are still loading
- **THEN** a loading spinner or progress indicator is drawn instead of a blank or half-drawn board
