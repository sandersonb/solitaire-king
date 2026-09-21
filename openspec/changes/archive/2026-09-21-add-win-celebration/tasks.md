## 1. Celebration model (anim.rs)

- [x] 1.1 Add a `FallingCard` struct (card, pos, vel, rot, ang_vel, g_scale, launched, at_rest) and a `Celebration` struct (falling cards, `started`, launch cursor) in `src/gui/anim.rs`, with tuning consts (`GRAVITY`, `REST`, `KICK`, `LAUNCH_INTERVAL`, `REST_EPS`, angular/`g_scale` ranges, `CELEBRATION_SECS = 6.0`). Verify `cargo build`.
- [x] 1.2 Implement `Celebration::start(foundations, rects, card_w, mobile_deck, sw, sh, now)` building the King→Ace, four-suits-per-rank release schedule seeded at each foundation's screen position (velocities zero, small random nudge). Verify with a unit test that the schedule contains all 52 cards ordered Kings-first to Aces-last.
- [x] 1.3 Implement `Celebration::update(dt, sw, sh, now)`: launch the next batch when due (`LAUNCH_INTERVAL`); apply gravity (`GRAVITY * g_scale`), integrate pos/rot; bounce off the floor (`vel.y *= -REST`, random horizontal kick + `ang_vel`) and off side walls (reflect `vel.x` with damping); settle (`at_rest`) when a floor contact leaves `|vel.y| < REST_EPS`. Clamp `dt` to a max (~1/30 s). Verify with a unit test that after enough simulated time all cards reach `at_rest` and stay within `[0, sw] × [.., sh]`.
- [x] 1.4 Add `Animator.celebration: Option<Celebration>` plus `active()`/`end()` and a cards accessor. Verify `cargo build`.

## 2. Rendering (render.rs)

- [x] 2.1 Add a rotation-capable card draw (extend `tex_params`/`draw_card` to take a `rotation` and draw about the card center). Verify `cargo build` and that existing (rotation = 0) draws are visually unchanged.
- [x] 2.2 In `board(..)`, when the celebration is active draw its falling cards on top of the board using the effective deck; when it is not active and the game is won, draw the win banner as today. Verify the banner no longer appears while the celebration is active.

## 3. Lifecycle & input (main.rs)

- [x] 3.1 Add a per-game win-phase flag; when a played win is first observed (`is_won() && !was_auto_solved()` and not yet started this game) call `celebration.start(...)` capturing foundation cards, rects, `card_w`, `mobile_deck`, and screen size. Reset the flag on new game. Verify a played win starts the celebration and a new game re-arms it.
- [x] 3.2 While the celebration is active, advance it each frame with `dt = get_frame_time()`, and on a pointer press **or** when `elapsed >= CELEBRATION_SECS` *finish* it (stop motion, show the banner, keep the settled cards drawn); give it top input priority so the ending press is swallowed (no board/undo/new action fires). Clear the settled cards only on a new game. Verify a click ends it early and ~6 s ends it on its own, both landing on the win banner with the cards still shown, and a new game clears them.
- [x] 3.3 Confirm an auto-solved finish does NOT start the celebration and still shows the auto-solved indication. Verify by auto-solving a deal.

## 4. Version bump & verification

- [x] 4.1 Bump the `Cargo.toml` patch version (0.2.1 → 0.2.2) and rebuild so `Cargo.lock` updates. Verify the new version shows on the splash.
- [x] 4.2 Run `cargo build`, `cargo clippy`, and `cargo test`; verify all pass with no new warnings. Build the WASM target.
- [x] 4.3 Smoke-test in a browser (via `tools/webcheck` or manually): win a deal, confirm the cascade plays ~6 s, cards fall fast / bounce / settle, a click ends it early, and the win banner shows afterward with correct score/time.
