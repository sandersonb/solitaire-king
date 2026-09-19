# gui-distribution Specification

## Purpose

How the browser build is produced and shipped: the WebAssembly artifact with its
static HTML page and runtime assets, and the continuous-deployment workflow that
publishes it to GitHub Pages so the game is playable from a public URL.

## Requirements

### Requirement: WebAssembly build
The GUI SHALL build to a WebAssembly artifact that runs in a modern browser, accompanied by a static HTML page that loads and starts it. Card and other assets SHALL be served alongside the page so the browser build loads them at runtime. The page SHALL show a loading indicator from the moment it opens until the application's first frame paints, so the download/startup wait is not a blank page.

#### Scenario: WASM artifact runs in a browser
- **WHEN** the WASM build and its HTML page and assets are served over HTTP and opened in a modern browser
- **THEN** the game renders and is playable

#### Scenario: Loading indicator during startup
- **WHEN** the page is opened and the WebAssembly is still downloading or starting
- **THEN** a loading indicator is visible until the application's first frame paints, at which point it is no longer shown

### Requirement: GitHub Pages deployment
The repository SHALL include a continuous-deployment workflow that builds the WebAssembly artifact and publishes it, with its page and assets, to GitHub Pages on pushes to the main branch, so the game is playable from a public URL.

#### Scenario: Push deploys the browser build
- **WHEN** a commit is pushed to the main branch
- **THEN** the workflow builds the WASM and publishes the page and assets to GitHub Pages

#### Scenario: Play-in-browser link
- **WHEN** a reader views the repository README
- **THEN** it links to the deployed GitHub Pages URL where the game can be played

### Requirement: Browser page suppresses native touch selection
The browser page hosting the WebAssembly build SHALL suppress the operating
system's native touch text-selection behaviors over the game canvas, so touch
interaction cannot trigger a text/select-all highlight, a selection callout, or a
tap-highlight flash. A tap, drag, or double-tap on the game SHALL be treated as
game input only and SHALL NOT visibly select page content.

#### Scenario: Touch does not select the page
- **WHEN** the player taps, double-taps, or drags on the game in a mobile browser
- **THEN** no text/select-all highlight, selection callout, or tap-highlight appears and the gesture only drives the game
