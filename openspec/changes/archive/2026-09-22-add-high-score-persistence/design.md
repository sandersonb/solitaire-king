## Context

See `proposal.md` — Why. Relevant existing structure:

- **Win seam.** `main.rs` already detects the first *played* win each game
  (`is_won() && !was_auto_solved() && !is_auto_solving() && !win_celebrated`) to
  start the celebration; at that point `elapsed_secs` is frozen and
  `session.final_score()` is final. This is the natural place to record a high
  score too, and `win_celebrated`/new-game reset logic already exists.
- **Banner.** `draw_win_banner` (render.rs) shows "You win!", the final score and
  time; it's the place to add the high score + new-high indicator.
- **No storage today.** No persistence crate, no `serde`. `macroquad::miniquad::date::now()`
  gives seconds-since-epoch on both targets (native `SystemTime`, web JS `Date`).
- **Web JS is vendored.** `web/mq_js_bundle.js` is copied by the deploy workflow and
  `tools/webcheck/build-dist.sh`; it already bundles the `sapp_jsutils` plugin.

## Goals / Non-Goals

**Goals:**
- Persist across sessions (browser storage on web, a file on native): a single best
  record (score, time, timestamp), two lifetime counters (wins, new games started),
  and the settings-dialog values.
- Load at startup; update+save on the relevant events (played win, new deal, setting
  change); ignore auto-solve for score/wins.
- Show the high score (and a new-high flash) on the win banner.
- Keep persistence best-effort and off the critical path; tolerate missing/corrupt.

**Non-Goals:**
- No leaderboard / top-N, no resume-in-progress, and no stats *screen* yet (the
  counters are stored but not surfaced this slice) — later slices of the epic.
- No `serde` (tiny delimited strings suffice).
- Not persisting per the native file's exact location beyond quad-storage's default.

## Decisions

### 1. Backend split behind `store.rs`
All access goes through a new `src/gui/store.rs` so the rest of the GUI never
touches the backend directly:

```
pub fn load(key: &str) -> Option<String>;   // None if absent/unreadable
pub fn save(key: &str, value: &str);         // best-effort
```

- **Web** (`cfg(target_arch = "wasm32")`): the `quad-storage` crate over the
  browser's Web Storage (`localStorage`).
- **Native** (`cfg(not(wasm))`): `std::fs`, one file per key under the user's config
  directory — `$XDG_CONFIG_HOME` else `$HOME/.config` (Unix/macOS), `%APPDATA%`
  (Windows), under a `klondike-solitaire/` subdir, created on first save. No crate
  needed (std only), so `quad-storage` is a **web-only dependency**
  (`[target.'cfg(target_arch = "wasm32")'.dependencies]`). This writes to `~/.config`
  rather than quad-storage's cwd `local.data`, and keeps the native dep surface at
  zero.

*Why the split (not quad-storage on both):* the user wants native data in the config
directory, which quad-storage's native backend doesn't offer; a few lines of
`std::fs` do, and it matches the "browser storage on web, fs otherwise" intent.

*Web plumbing:* vendor `web/quad-storage.js`, add `<script src="quad-storage.js">`
in `index.html` after `mq_js_bundle.js` and before `load("…wasm")`; add copy steps
to `deploy-pages.yml` and `tools/webcheck/build-dist.sh`; note the new vendored JS
in `CLAUDE.md`. `sapp_jsutils` JS is already in the bundle, so only
`quad-storage.js` is added (verify at apply; add `sapp_jsutils.js` only if needed).

*Compatibility fallback:* `quad-storage` is 0.1.x; if it does not build against
miniquad 0.4, implement the web side of `store.rs` by hand via a small
`sapp-jsutils`-based `localStorage` binding (the JS is already bundled). Only the
web arm of `store.rs`, `Cargo.toml`, and the vendored JS change; callers and the
native arm are unaffected.

### 2. Persisted data and serialization
Each record is a small struct with a `parse`/`to_field` pair over a delimited
string, stored under its own versioned key (the `.v1` leaves room to evolve).
Parsing is tolerant: any missing/garbled field yields a default (→ treated as no
record / defaults).

```
#[derive(Clone, Copy)]
pub struct HighScore { pub score: i64, pub time_secs: u64, pub achieved_at: i64 }
// key "highscore.v1"  ->  "<score>|<time_secs>|<achieved_at>"

#[derive(Clone, Copy, Default)]
pub struct Counters { pub wins: u64, pub new_games: u64 }
// key "stats.v1"      ->  "<wins>|<new_games>"

// Settings persistence: the four dialog values.
// key "settings.v1"   ->  "<draw_three:0|1>|<deck:a|m|s>|<solver:0|1>|<seed:0|1>"
```

`achieved_at` is `miniquad::date::now() as i64` (unix seconds). The deck field
encodes the `Option<bool>` override: `a` = Auto (`None`), `m` = Mobile
(`Some(true)`), `s` = Standard (`Some(false)`).

### 3. Lifecycle in the main loop
Load once at startup: `high_score` (`Option<HighScore>`), `counters`, and the
persisted `settings` (applied over the config/args defaults), plus a per-game
`new_high = false`.

- **High score + wins:** at the existing first-played-win seam, increment
  `counters.wins` and save; if `high_score.is_none_or(|h| final > h.score)`, build
  the record from `session.elapsed_secs()` and the current timestamp, save, set
  `high_score = Some(rec)` and `new_high = true`. Strictly-greater only, so ties
  keep the older record. `elapsed_secs` is frozen and `final_score()` final here,
  and the once-per-win guard prevents double counting; auto-solve is already
  excluded by that seam's condition.
- **New-games counter:** increment `counters.new_games` and save wherever a fresh
  deal is created — the initial `Session::new` at startup *and* `new_game()` (a
  single helper both call, so every new-game entry point counts once).
- **Settings:** load at startup and apply as the starting `Settings` (on native,
  an explicit `--draw` etc. launch arg still wins for that launch); on any change
  in `handle_settings`, save the settings blob immediately.
- Reset `new_high = false` in the same `!is_won()` branch that already clears
  `win_celebrated`.

### 4. Display
`board(...)` gains `high: Option<HighScore>` and `new_high: bool`, forwarded to
`draw_win_banner`, which adds a "High score: N" line and, when `new_high`, a "New
high score!" flourish. `HighScore` is `Copy`, so passing it by value is cheap. The
stored timestamp isn't shown yet (kept for later slices); only the score is
displayed now.

## Risks / Trade-offs

- [`quad-storage` incompatible with miniquad 0.4] → `store.rs` boundary lets us swap
  to the hand-rolled impl without touching callers (see Decision 1).
- [Another vendored JS file to keep in sync] → Documented in `CLAUDE.md` alongside
  `mq_js_bundle.js`; re-vendor if the storage stack is bumped.
- [Config dir not resolvable on native (no `HOME`/`APPDATA`)] → `store.rs` returns
  `None`/no-ops (best-effort); the game still runs, just without native persistence.
- [Storage failure mid-save] → Best-effort by design; a failed save just means the
  record isn't kept, never a crash (save returns nothing; load tolerates absence).
- [Score can be negative] → `i64` and strict-greater comparison handle it; the first
  win always sets the record regardless of sign.

## Open Questions

None blocking. Displaying the timestamp/date and a fastest-time stat are natural
follow-on slices and don't change this layer or its API.
