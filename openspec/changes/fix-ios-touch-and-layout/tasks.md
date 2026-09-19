## 1. Deck selection decoupled from touch (issue 5)

- [x] 1.1 Add a `MOBILE_DECK_MAX` constant (~960 logical px) and a `mobile_deck: bool` field to `Layout` in `src/gui/layout.rs`, set from `sw <= MOBILE_DECK_MAX` (independent of the `mobile`/`touch` flag). Verify `cargo build` succeeds and the field is populated in `Layout::compute`.
- [x] 1.2 In `src/gui/render.rs`, pass `layout.mobile_deck` (not `layout.mobile`) into `assets.face(..)` / `draw_card` / `draw_run` deck selection. Verify a wide desktop window shows the standard deck and a narrow (<960) window shows the mobile deck.
- [ ] 1.3 Manually verify on an iPhone-class width (~874) the mobile deck loads and on an iPad-class width (>1133) the standard deck loads, per the gui-rendering scenarios "Mobile card art preferred on touch" and "Standard deck on larger viewports regardless of touch".
- [x] 1.4 Add a session-only `deck_override: Option<bool>` to the `Settings` struct in `src/gui/main.rs` (None=Auto, Some=forced) and resolve the effective deck as `deck_override.unwrap_or(layout.mobile_deck)` where `assets.face(..)` is called. Verify `cargo build` and that with override unset behavior matches task 1.2.
- [x] 1.5 Add a deck control to the settings dialog (render in `src/gui/render.rs`, hit-handling where the other settings toggles are handled) that cycles Auto → Mobile → Standard and shows the current state; apply it immediately to the current game. Verify by toggling in a browser: cards switch set immediately and Auto restores the width-based choice — gui-shell scenarios "Deck override takes effect immediately" and "Deck Auto follows the viewport", and gui-rendering scenario "Settings dialog is drawn".

## 2. Touch drop matches the drawn card (issue 1)

- [x] 2.1 Promote the drag lift amount (currently `card_w * 0.9` in `src/gui/render.rs`) to a single shared constant/helper usable by both render and input, so draw and hit-test cannot drift. Verify `cargo build` and that render still uses the shared value.
- [x] 2.2 In `src/gui/input.rs`, resolve the drop by hit-testing `nearest_pile` at the drawn-card reference point (pointer shifted up by the lift on touch), not the raw pointer. Verify with `cargo test` (existing tests pass) and that on desktop/mouse behavior is unchanged (lift = 0).
- [ ] 2.3 Manually verify on iOS that releasing a dragged card when it *visually* overlaps a pile registers the move, matching desktop's forgiving drop (gui-input scenario "Touch drop matches the visible card").

## 3. Fan-aware card sizing (issue 2)

- [x] 3.1 In `src/gui/layout.rs`, add `SIZING_FAN_FRAC` (~0.18) and use it (instead of `MIN_FAN_FRAC`) in the height-budget `col_span` so cards are sized to a comfortable fan; keep `MIN_FAN_FRAC` as the `fan_dy` clamp floor. Verify `cargo build` and that on a wide/short window a tall column shows a larger fan (more of each card visible) while portrait/narrow is unchanged.
- [ ] 3.2 Manually verify on a maximized landscape iPad that a tall tableau column keeps readable vertical spacing (rank/suit corner visible) — gui-rendering scenario "Short viewport keeps a readable fan".

## 4. Smaller touch pick-up zoom (issue 3)

- [x] 4.1 In `src/gui/render.rs`, reduce the touch drag `scale` from `1.15` to `≈1.06` (keep the vertical lift). Verify the dragged card is only modestly enlarged on touch and native/mouse is unchanged — gui-rendering scenario "Touch pick-up is a subtle cue".

## 5. Suppress native touch selection (issue 4)

- [x] 5.1 In `web/index.html`, add `-webkit-user-select:none; user-select:none; -webkit-touch-callout:none; -webkit-tap-highlight-color:transparent; touch-action:none;` to `html, body` and the `#glcanvas` canvas rules. Do NOT modify `mq_js_bundle.js`. Verify the CSS is present and the page still loads.
- [x] 5.2 Manually verify on iOS Safari that tapping/double-tapping/dragging the game never produces the blue select-all highlight or a selection callout — gui-distribution scenario "Touch does not select the page".

## 6. Build & regression verification

- [x] 6.1 Run `cargo build`, `cargo clippy`, and `cargo test`; verify all pass with no new warnings.
- [x] 6.2 Build the WASM target and smoke-test in a desktop browser (drop feel, deck at various widths, aspect cap by resizing) before device QA; verify no regression from the current deployed behavior.
