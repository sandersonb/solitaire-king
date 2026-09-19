## ADDED Requirements

### Requirement: Browser page suppresses native touch selection
The browser page hosting the WebAssembly build SHALL suppress the operating
system's native touch text-selection behaviors over the game canvas, so touch
interaction cannot trigger a text/select-all highlight, a selection callout, or a
tap-highlight flash. A tap, drag, or double-tap on the game SHALL be treated as
game input only and SHALL NOT visibly select page content.

#### Scenario: Touch does not select the page
- **WHEN** the player taps, double-taps, or drags on the game in a mobile browser
- **THEN** no text/select-all highlight, selection callout, or tap-highlight appears and the gesture only drives the game
