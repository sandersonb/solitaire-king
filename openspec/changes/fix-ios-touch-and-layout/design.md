## Context

See `proposal.md` — Why. All five issues are browser/touch-only; native desktop is
unaffected. Key mechanics established while investigating:

- **Logical vs physical pixels.** `main.rs` sets `high_dpi: true`. The miniquad
  glue sizes the framebuffer to `clientWidth * devicePixelRatio` (physical), but
  macroquad's `screen_width()`/`screen_height()` — what `Layout::compute` consumes
  — are in *logical* CSS points. iPhone 17 (`2622×1206` physical, DPR 3) presents
  as **`874×402`** logical landscape. This is also why `input.rs` divides
  `touches()` by `screen_dpi_scale()`: touches arrive physical, layout is logical.
- **`mobile` flag conflation.** `Layout::compute(sw, sh, _, touch)` computes
  `mobile = touch || sw < sh || sw < 700` and this single boolean currently drives
  three things: the mobile UI profile (control-bar height), the drag lift/zoom
  (`render.rs`), and the detailed-deck selection (`assets.face(.., mobile)`).
- **Touch drop mismatch.** On touch the dragged run is drawn at `top_left - lift`
  (`lift = card_w * 0.9`) and scaled `1.15×`, but `nearest_pile` hit-tests the raw
  pointer `(x, y)`. The card visually sits ~0.9 card-widths above the finger, so a
  drop that looks on-target misses by the lift.
- **Landscape squish.** On a wide/short viewport, `card_w` is bound by the height
  budget yet `fan_dy` clamps to `MIN_FAN_FRAC` (0.10·card_h); cards stack almost
  flush and suits vanish.

## Goals / Non-Goals

**Goals:**
- Drop feel on touch matches desktop's forgiving drop zones.
- Readable tableau fan on maximized landscape iPad.
- Subtle (not oversized) touch pick-up.
- No native page selection / blue highlight on touch.
- Detailed deck chosen by logical width, decoupled from touch, pivoting just above
  an iPhone 17 (~960 logical px).

**Non-Goals:**
- No change to native desktop behavior, the domain/rules/solver, or the deploy
  workflow.
- No re-vendoring of `mq_js_bundle.js` (only `web/index.html` CSS changes).
- Not reworking the mobile UI profile (control bar) or asset pipeline.

## Decisions

### 1. Deck selection split from the mobile-UI flag (issue 5)
Introduce a separate signal for deck art based purely on logical viewport width,
leaving the touch-driven `mobile` flag for UI/drag behavior.

- Add a `detailed_deck: bool` field to `Layout` set from `sw <= DETAILED_DECK_MAX`
  where `DETAILED_DECK_MAX ≈ 960` (just above iPhone 17 Pro Max's ~956 logical
  landscape width, safely below the smallest iPad's ~1133). Compare on width so a
  phone in landscape (874/956) still gets the detailed deck.
- `render.rs` passes `layout.detailed_deck` (not `layout.mobile`) into
  `assets.face(rank, suit, ..)` and `draw_card`/`draw_run`.
- `mobile` keeps driving the control-bar sizing and the drag lift/zoom.

*Why width, not touch:* "pivot just larger than iPhone 17" is only meaningful as a
resolution rule. Touch-forcing (`touch_seen`) would keep every iPad detailed,
which is the opposite of the requested pivot. *Alternative considered:* keep the
touch force and only bump 700→960 — rejected because iPads stay detailed, so the
pivot wouldn't actually engage (chosen approach confirmed with the user).

*Trade accepted (user-approved):* large touch tablets now use the standard deck —
mitigated by the manual override below.

### 1b. Settings deck override (session-only, immediate)
Add a session-only override so a mis-guessed auto default is recoverable. Model it
as a tri-state on the `Settings` struct in `main.rs`, e.g.
`deck_override: Option<bool>` — `None` = Auto (use `layout.detailed_deck`),
`Some(true)` = force detailed, `Some(false)` = force standard. The settings dialog
gains a control that cycles Auto → Detailed → Standard. Deck art is cosmetic, so
unlike draw mode it applies to the *current* game immediately. The effective signal
handed to `assets.face(..)` is `deck_override.unwrap_or(layout.detailed_deck)`.

*Why tri-state, not a plain on/off:* Auto must remain distinct from an explicit
choice so the deck stays responsive to rotation/resize until the player opts out.
*Why session-only now:* real persistence (localStorage plugin + native fallback) is
being plumbed with high scores in a later story; a session-only toggle needs no new
JS glue and keeps the vendored bundle clean. It reverts to Auto on reload, which is
acceptable until persistence lands. *Alternative — persist via localStorage now:*
deferred to avoid introducing the first custom JS↔WASM binding in this cleanup
release.

### 2. Touch drop resolved from the drawn card position (issue 1)
Compute the drop point by shifting the pointer by the same lift used when drawing:
resolve `nearest_pile` at `(x, y - lift)` on touch (and, if it matters, bias by the
grab offset so the card's reference corner, not the finger, is tested). Keep the
existing generous `pad` in `nearest_pile`. Lift is currently a `render.rs` local
(`card_w * 0.9`); promote it to a shared constant (e.g. `layout` or an `input`
const) so draw and hit-test cannot drift apart.

*Why:* the drop zones aren't actually too small — the reference point is wrong.
Aligning the hit-test point to the drawn card reuses the desktop `pad` and makes
touch match desktop. *Alternative:* just enlarge `pad` on touch — rejected as it
papers over the offset and would make adjacent piles ambiguous.

### 3. Fan-aware card sizing (issue 2)
The height budget sizes cards so the tallest column fits *at the minimum fan*:
`col_span = 2 + MIN_FAN_FRAC·(n−1)` with `MIN_FAN_FRAC = 0.10`. For a tall column
on a short viewport this forces the fan to that 0.10 floor by construction, hiding
suits. Fix: size cards to a **comfortable** fan by using a larger fraction in the
sizing budget — `col_span = 2 + SIZING_FAN_FRAC·(n−1)` with `SIZING_FAN_FRAC ≈
0.18` — so `height_card_h` (and thus `card_w` when height-bound) shrinks enough
that the later `fan_dy` lands around the comfortable value instead of the floor.
Keep `MIN_FAN_FRAC` as the absolute `fan_dy` clamp floor for extreme cases. This
is a one-line change to the sizing fraction; `max_col_len` is already recomputed
per frame and already drives card size, so behavior stays adaptive with no new
resize jank.

*Why not the earlier aspect-cap/pillarbox:* investigation showed cards are
**height-bound** on a landscape iPad (`card_w = min(width, height)` → height wins),
so clamping width changes neither the card size nor the fan; and real iPad
viewports (~1.4–1.7 aspect, even with Safari chrome) never reach a ~1.9 cap. The
squish is a vertical-budget problem, so the vertical budget is the right lever.
*Alternative — cap card height directly:* equivalent effect via a blunter knob;
the fan-fraction tweak is chosen because it ties the shrink to the actual overlap
comfort and touches the existing formula minimally. *Optional add-on (not taken
now):* a mild ~1.6 aspect cap purely for ultrawide desktop monitors.

### 4. Reduce touch pick-up scale (issue 3)
Change the drag scale in `render.rs` from `1.15` to `≈1.06`. Keep the vertical
lift (so the finger doesn't occlude the card). Single-constant change.

### 5. Suppress native selection via page CSS (issue 4)
In `web/index.html`, add to `html, body, canvas` (and `#glcanvas`):
`-webkit-user-select: none; user-select: none; -webkit-touch-callout: none;
-webkit-tap-highlight-color: transparent; touch-action: none;`. The glue already
`preventDefault`s touch events; the CSS closes the gap for selection/callout/
highlight that `preventDefault` alone doesn't cover on iOS Safari.

*Why CSS only:* keeps `mq_js_bundle.js` unmodified (the CLAUDE.md re-vendor patch
list is unaffected). `touch-action: none` also prevents double-tap-zoom / scroll
gestures from stealing input.

## Risks / Trade-offs

- [Large iPads switch to the standard deck] → User-approved; standard art is fully
  legible at tablet size. Threshold (~960) leaves a clean gap to iPad's ~1133.
- [Lift constant duplicated between draw and hit-test drifts again] → Promote it to
  one shared constant used by both `render.rs` and `input.rs`.
- [`touch-action: none` disables page scroll on the canvas] → Intended; the game
  fills the viewport and manages its own input, and `overflow: hidden` is already
  set on `html, body`.
- [Comfortable-fan sizing makes cards smaller when a tall column exists] →
  Intended and adaptive: cards shrink only while a column is tall enough to need
  it (per-frame `max_col_len`), and shrink slightly, to reveal each card's corner.
  Portrait/narrow viewports are width-bound so they are unaffected.
- [`screen_width()` logical-vs-physical assumption is load-bearing] → Already
  relied on by the existing touch-coordinate divide; verify visually on a real iOS
  device during QA.

## Open Questions

None blocking. Exact numeric constants (`DETAILED_DECK_MAX`, `MAX_ASPECT`, drag
scale) are set to the values agreed with the user and can be nudged during device
testing without changing the specs or task breakdown.
