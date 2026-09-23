## Why

Nothing the player achieves survives a reload — the game has no memory. This is
the first slice of the persistence epic: keep a **high score** (with the time and
timestamp of the game that set it) across sessions, laying the storage foundation
that later slices (leaderboard, resume-in-progress, persisted settings) build on.

## What Changes

- **Cross-platform local persistence.** Add a small persistence layer that stores
  and loads key-value data across launches: **browser storage on web** and a
  **local file on native**. Missing or unreadable data SHALL be tolerated (treated
  as "no data"), never crashing the game.
- **Persisted high-score record.** Track the single best result achieved by play:
  the highest final score, the elapsed **time** of that game, and a **timestamp**
  of when it was set. The record SHALL be loaded at startup and updated (and saved)
  whenever a played win beats the stored score. Auto-solved finishes (not scored
  wins) SHALL NOT affect it.
- **Show the high score on the win banner.** The win banner SHALL display the
  persisted high score, and SHALL indicate when the just-finished game set a new
  high score.
- **Persisted play counters.** Track two lifetime counters — games won (by play)
  and new games started — incremented as those events occur and accumulated across
  sessions. (Stored now; surfacing them in a stats view is a later slice.)
- **Persisted settings.** The settings-dialog values (draw mode, deck
  Auto/Mobile/Standard, background-solver toggle, show-seed) SHALL be saved when
  changed and restored on the next launch, replacing the previous session-only
  behavior.
- **Version bump.** Increment the crate patch version.

## Capabilities

### New Capabilities
- `gui-persistence`: local persistence of small data across sessions (browser
  storage on web, a file on native): the high-score record (score + time +
  timestamp), lifetime play counters (wins, new games started), and the persisted
  settings — all loaded at startup and saved as they change.

### Modified Capabilities
- `gui-shell`: on a played win the high-score record is updated/persisted when
  beaten, and the win banner shows the high score and flags a new high score; the
  settings dialog values are now persisted across launches (previously
  session-only).

## Impact

- New `src/gui/store.rs` — a thin persistence wrapper (web: `quad-storage` browser
  storage; native: `std::fs` files under the user's config dir, e.g.
  `~/.config/klondike-solitaire/`) and the persisted records (`HighScore`, the play
  counters, and the settings) with string (de)serialization; tolerant of
  absent/corrupt values.
- `main.rs` — load the record, counters, and settings at startup; on a played win
  update+save the high score (if beaten) and increment wins; increment the
  new-games counter on each deal; save settings when a dialog value changes.
- `src/gui/render.rs` — win banner shows the high score and the new-high indicator.
- `Cargo.toml` — add `quad-storage` as a **web-only** dependency (native uses
  `std::fs`); patch version bump.
- `web/index.html` + `web/quad-storage.js` (vendored) — load the storage JS shim
  after `mq_js_bundle.js`; `.github/workflows/deploy-pages.yml` and
  `tools/webcheck/build-dist.sh` copy it into `dist/`. `CLAUDE.md` note updated for
  the new vendored JS.
- Timestamp uses `macroquad::miniquad::date::now()` (no new dependency). No changes
  to the domain, rules, solver, or CLI.
