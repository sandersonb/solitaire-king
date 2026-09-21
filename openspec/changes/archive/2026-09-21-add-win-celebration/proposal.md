## Why

Winning currently just dims the board and shows a banner — anticlimactic for the
payoff moment. A physical, chaotic card-cascade celebration (in the spirit of the
classic Windows Solitaire win) makes a completed game feel rewarding.

## What Changes

- **Win celebration animation.** On a win reached by play, the 52 cards SHALL
  launch from the four foundation piles as falling sprites — starting with the
  Kings and proceeding down through the ranks to the Aces — and animate under
  gravity: accelerating as they fall (with a touch of random acceleration),
  bouncing off the bottom of the screen with a random horizontal kick and a bit of
  rotation, bouncing possibly several times, and eventually settling. The result is
  deliberately chaotic, scattering cards across the screen before they come to
  rest. As each card launches, the next card still on its foundation is shown resting
  in the slot, so each pile drains from King to empty. The whole sequence SHALL run
  for about 6 seconds (so cards fall quickly).
- **Dismiss to the win banner.** A click/tap SHALL end the celebration immediately;
  when it ends (by click or after ~6 s) the existing win banner (final score and
  elapsed time) SHALL be shown. Either path reaches the same end-state. The settled
  cards SHALL remain on screen beneath the banner and are cleared only on a new game.
- **Auto-solved finishes are unchanged.** A win reached by auto-solve (explicitly
  "not a scored win") SHALL NOT play the celebration; it keeps its current
  auto-solved indication.
- **Version bump.** Increment the crate patch version.

## Capabilities

### New Capabilities
<!-- None: extends existing GUI capabilities. -->

### Modified Capabilities
- `gui-animation`: add a win-celebration animation mode (physics-based falling and
  bouncing cards), distinct from the existing point-to-point card tweens.
- `gui-shell`: on a played win the celebration plays first and is dismissible; the
  win banner is shown when the celebration ends (by click or timeout).

## Impact

- `src/gui/anim.rs` — a celebration state/particle model (per-card position,
  velocity, rotation, angular velocity; gravity + jitter; floor bounce with
  restitution and random horizontal/rotational kick; settle). Uses `macroquad`'s
  `rand` (`quad-rand`, already a transitive dep) — no new dependency.
- `src/gui/render.rs` — draw the falling/bouncing cards (rotated sprites via
  `DrawTextureParams.rotation`), and gate the win banner on the celebration being
  finished.
- `src/gui/main.rs` — start the celebration when a played win is first detected;
  advance it each frame with the frame delta; end it on click or at ~20 s.
- `Cargo.toml` — patch version bump.
- No changes to the domain, rules, solver, CLI, or asset pipeline. Native and web
  behave the same.
