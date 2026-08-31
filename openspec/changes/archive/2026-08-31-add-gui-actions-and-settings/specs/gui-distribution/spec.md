## MODIFIED Requirements

### Requirement: WebAssembly build
The GUI SHALL build to a WebAssembly artifact that runs in a modern browser, accompanied by a static HTML page that loads and starts it. Card and other assets SHALL be served alongside the page so the browser build loads them at runtime. The page SHALL show a loading indicator from the moment it opens until the application's first frame paints, so the download/startup wait is not a blank page.

#### Scenario: WASM artifact runs in a browser
- **WHEN** the WASM build and its HTML page and assets are served over HTTP and opened in a modern browser
- **THEN** the game renders and is playable

#### Scenario: Loading indicator during startup
- **WHEN** the page is opened and the WebAssembly is still downloading or starting
- **THEN** a loading indicator is visible until the application's first frame paints, at which point it is no longer shown
