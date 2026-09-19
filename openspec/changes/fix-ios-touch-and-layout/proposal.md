## Why

The browser build plays well on desktop but has several iOS-specific defects that
make it frustrating on iPhone and iPad: drops require near-pixel-perfect aim,
cards squish together vertically on a maximized landscape iPad until suits are
unreadable, the picked-up card balloons too large, and the OS occasionally
selects the whole page (blue highlight). This is a second cleanup release
targeting those touch/mobile issues.

## What Changes

- **Drop zones match desktop on touch.** On touch, the dragged card is drawn
  lifted above the finger, but the drop is hit-tested at the finger, so the card
  visually overlaps a pile while the release misses it. The drop point SHALL be
  computed from where the card is *drawn* (compensating for the lift), so landing
  the card on a pile registers — matching desktop's forgiving feel.
- **Fan-aware card sizing so the tableau fan stays readable.** The tallest column
  currently drives card size assuming the *minimum* fan spacing, so on a short
  (landscape) viewport the fan always collapses to that floor and suits are hidden.
  Cards SHALL instead be sized to a *comfortable* fan spacing, so more of each
  overlapped card (its rank/suit corner) shows; cards get slightly smaller on tall
  columns rather than overlapping to the minimum. (Investigation showed the earlier
  aspect-cap/pillarbox idea does not fix this: on a landscape iPad cards are
  height-bound, so capping width leaves the fan unchanged.)
- **Smaller pick-up enlargement on touch.** Reduce the dragged-card scale from
  1.15× to ~1.06× so the lifted card is a subtle cue, not oversized.
- **Suppress native mobile text selection.** The web page SHALL disable
  touch text selection, the touch callout, and the tap highlight so a touch or
  double-tap can't select-all / flash the page blue.
- **Detailed deck pivots by viewport width, decoupled from the touch flag.** The
  higher-legibility "mobile" deck SHALL be chosen by logical viewport width
  (phone-sized, threshold just above an iPhone 17's ~960 logical px), independent
  of whether the device is touch. Phones get the detailed deck; larger tablets and
  desktops get the standard deck. The mobile *UI* profile (control bar, drag
  lift, drop-zone handling) stays driven by touch/portrait/narrow as today.
- **Settings deck override.** The settings dialog SHALL add a deck control with
  three states — Auto (the width-based default above), Detailed, and Standard — so
  the player can override the auto pick if it guesses wrong for their device. The
  override applies immediately and is session-only (real persistence is deferred
  to a later change).

## Capabilities

### New Capabilities
<!-- None: all changes refine existing GUI behavior. -->

### Modified Capabilities
- `gui-input`: drop-zone hit-testing on touch compensates for the drawn card's
  lift so the effective drop point matches where the card appears.
- `gui-rendering`: (a) board layout sizes cards to a comfortable tableau fan so
  suits stay readable on short/landscape viewports; (b) the touch pick-up
  enlargement is reduced; (c) the
  detailed card-image set is selected by logical viewport width, decoupled from
  the touch/mobile-UI profile, unless overridden in settings; (d) the settings
  dialog renders the deck control.
- `gui-shell`: the settings dialog gains a session-only deck override
  (Auto / Detailed / Standard) applied immediately.
- `gui-distribution`: the browser page suppresses native touch text selection,
  callout, and tap-highlight.

## Impact

- `src/gui/input.rs` — `nearest_pile` drop-point adjustment for the touch lift.
- `src/gui/layout.rs` — size cards to a comfortable fan spacing (not the minimum);
  split the deck-selection signal (logical width) from the touch/mobile-UI
  `mobile` flag.
- `src/gui/render.rs` — reduced drag scale; use the effective deck signal
  (width default, or the settings override) for `assets.face(..)`; render the deck
  control in the settings dialog.
- `src/gui/assets.rs` — `face(..)` deck-selection call sites (if the signal name
  changes).
- `src/gui/main.rs` — add a session-only deck-override field to the `Settings`
  struct and resolve the effective deck (override else width default).
- `web/index.html` — CSS to disable user-select / touch-callout / tap-highlight.
- No changes to the domain, rules, solver, or CLI. Native behavior is unaffected.
