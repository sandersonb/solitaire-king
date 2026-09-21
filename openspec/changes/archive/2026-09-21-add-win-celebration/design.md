## Context

See `proposal.md` — Why. Relevant existing structure:

- **Win detection.** The main loop already knows `session.is_won()` and
  `session.was_auto_solved()`; on a win it currently draws `draw_win_banner`
  immediately (render.rs), with the control bar on top so New/Settings stay usable.
- **Animation.** `anim.rs` holds point-to-point `CardAnim` tweens driven from
  `get_time()`. The celebration is a *different* kind of motion (per-card physics),
  so it warrants its own state rather than reusing `CardAnim`.
- **Win state = all 52 on foundations.** `session.state.foundations: [Foundation; 4]`
  each hold A→K; `layout.foundations: [Rect; 4]` give their screen positions and
  `layout.card_w` the size. All foundation cards are face-up.
- **Randomness.** `macroquad::rand` (the `quad-rand` crate) is already pulled in
  transitively and works on wasm — `gen_range(a, b)` needs no new dependency.
- **Rotation.** `DrawTextureParams` has a `rotation` field (radians); the current
  `tex_params` helper doesn't set it.

## Goals / Non-Goals

**Goals:**
- A chaotic, physical card cascade on a played win, ~6 s, Kings→Aces.
- Click/tap ends it; either path ends at the existing win banner.
- Frame-rate independent (advance by frame delta), native and web identical.
- No new dependencies; purely cosmetic (no state/scoring impact).

**Non-Goals:**
- Not a general particle/physics engine — just enough for cards.
- No change to auto-solve finishes (no celebration there).
- No inter-card collisions (cards pass over each other; only floor/walls collide).

## Decisions

### 1. A `Celebration` model in `anim.rs`
Add a `Celebration` struct: a `Vec<FallingCard>` plus a `started: f64` timestamp
and a launch cursor. Each `FallingCard` holds `card: Card`, `pos: Vec2`,
`vel: Vec2`, `rot: f32`, `ang_vel: f32`, `g_scale: f32` (per-card gravity jitter),
`launched: bool`, and `at_rest: bool`. `Animator` gains
`celebration: Option<Celebration>`. Methods: `start(...)`, `update(dt, sw, sh)`,
`active()`, `end()`, and an accessor for the cards to draw.

*Why here:* `anim.rs` owns motion; the spec frames this as an animation mode.
*Alternative:* a struct in `main.rs` — rejected; keeps physics out of the loop.

### 2. Launch order and cadence (Kings → Aces)
On `start`, build the release schedule: rank order King, Queen, … Ace; for each
rank the four suits (one per foundation) are queued, each seeded at its
foundation's screen position. `update` "launches" the next batch when
`elapsed >= next_launch`, stepping `next_launch` by `LAUNCH_INTERVAL` (~0.25 s), so
13 ranks finish launching in ~3–4 s while earlier cards are already bouncing. A
launched card starts with near-zero velocity (small random downward + horizontal
nudge) so it "drops directly down" first.

### 3. Per-card physics (gravity, bounce, settle)
Each active card each frame:
- `vel.y += GRAVITY * g_scale * dt` (GRAVITY ≈ 2600 px/s²; `g_scale` ≈
  `gen_range(0.85, 1.15)` fixed per card) — fast fall so the whole thing fits ~6 s.
- `pos += vel * dt`; `rot += ang_vel * dt`.
- **Floor** (`pos.y + card_h >= sh`): clamp to floor, `vel.y = -vel.y * REST`
  (REST ≈ `gen_range(0.45, 0.7)`), add a random horizontal kick
  (`vel.x += gen_range(-KICK, KICK)`) and random `ang_vel` so it tumbles off in a
  new direction. Multiple bounces decay via REST < 1.
- **Walls** (left/right edges): reflect `vel.x` with light damping so cards stay
  on-screen ("all sorts of places" but bounded).
- **Settle:** after a floor contact with `|vel.y| < REST_EPS` (~60 px/s), set
  `at_rest`, zero velocities and `ang_vel`, leaving the final `rot`.

All tuning constants (`GRAVITY`, `REST`, `KICK`, `LAUNCH_INTERVAL`, `REST_EPS`,
angular ranges) are module consts, nudgeable during QA without spec changes.

### 4. Lifecycle in the main loop
Track a small win-phase state so the celebration starts once and the banner
follows. When a played win is first observed (`is_won() && !was_auto_solved()` and
no celebration yet started this game), call `celebration.start(...)` capturing the
foundation cards, their rects, `card_w`, the effective deck (`mobile_deck`), and
current screen size. Each frame while active: `update(dt, sw, sh)` with
`dt = get_frame_time()`; on a pointer press **or** `elapsed >= CELEBRATION_SECS`
(6 s) the animation *finishes* — motion stops and the banner shows, but the settled
cards stay drawn (a `finished` flag; `celebration_active()` is false, yet
`celebration()` still yields the cards). A new game (`!is_won()`) both resets the
once-per-win guard and clears the cascade (`end_celebration()`), so the cards are
removed only then and a later win celebrates again.

*Input gating:* while the celebration is active it has top input priority — a
press ends it and is otherwise swallowed (no board/undo/new action fires). Once
ended, the banner + control bar behave exactly as today.

### 5. Rendering
Draw the board painting each foundation's **next not-yet-launched card** while
the celebration is active (a "draining" deck, via `Celebration::resting_top`),
then overlay the falling cards on top — so a card never falls from an empty slot
and the King doesn't sit in its slot for the whole animation. The cascade cards are
drawn whenever a `celebration()` is present (playing *or* finished); the banner and
control bar are drawn only when the celebration is **not actively playing**, so on
finish the banner appears over the still-drawn settled cards (dimmed by the banner's
existing overlay, at the bottom of the screen — the "You win!" text sits above). No
dim during active play. Add a rotation-capable card draw that renders a **rounded**
rotated body + border (matching the resting cards' `corner_radius` of `w*0.09`, via
a rotated analogue of `round_rect`) with the face sprite rotated on top
(`DrawTextureParams.rotation`). Faces use the same deck selection as the board; the
procedural fallback may draw upright if rotating it is awkward.

## Risks / Trade-offs

- [Cards still bouncing at the 6 s cap] → Acceptable: at 6 s the animation ends
  and the banner shows regardless; REST < 1 means most settle well before then.
- [Resize mid-celebration] → Floor/walls recompute from current `sw/sh` each frame;
  positions are absolute px, so a resize just relocates the floor — no crash, minor
  visual jump at worst.
- [Procedural fallback without rotation looks flat] → Only when card sprites are
  absent (rare in the deployed build); faces normally rotate.
- [Frame-delta spikes (tab backgrounded)] → Clamp `dt` to a max (e.g. 1/30 s) so a
  long stall doesn't teleport cards through the floor.

## Open Questions

None blocking. Physics constants are tuned by feel during device QA (the design
lists starting values) and don't affect the specs or task breakdown.
