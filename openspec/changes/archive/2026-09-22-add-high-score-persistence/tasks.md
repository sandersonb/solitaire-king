## 1. Persistence layer (store.rs + deps)

- [x] 1.1 Add `quad-storage` to `Cargo.toml` as a web-only dependency (`[target.'cfg(target_arch = "wasm32")'.dependencies]`) and verify `cargo build` (native) and `cargo build --target wasm32-unknown-unknown` both succeed. If it does not build against miniquad 0.4, switch the web arm to the hand-rolled `sapp-jsutils` localStorage fallback behind the same `store.rs` API — see design Decision 1.
- [x] 1.2 Create `src/gui/store.rs` with `load(key) -> Option<String>` and `save(key, value)` and a `cfg` split: web → `quad-storage` (browser storage); native → `std::fs` one file per key under the config dir (`$XDG_CONFIG_HOME` else `$HOME/.config`, `%APPDATA%` on Windows, `klondike-solitaire/` subdir, created on first save). Absent/unreadable reads return `None`; saves are best-effort (never panic; unresolved config dir is a no-op). Add `mod store;` in `main.rs`. Verify both targets build.
- [x] 1.3 Add the persisted records in `store.rs`, each with `parse()`/`to_field()` and load/save helpers on versioned keys: `HighScore { score: i64, time_secs: u64, achieved_at: i64 }` (`highscore.v1`, `"<score>|<time>|<ts>"`), `Counters { wins: u64, new_games: u64 }` (`stats.v1`, `"<wins>|<new_games>"`), and a settings blob (`settings.v1`, `"<draw:0|1>|<deck:a|m|s>|<solver:0|1>|<seed:0|1>"`). Verify with unit tests that each round-trips and that malformed/empty strings fall back to default/`None`.

## 2. Web JS plumbing (quad-storage shim)

- [x] 2.1 Vendor `web/quad-storage.js` (version-matched) and add `<script src="quad-storage.js"></script>` in `web/index.html` after `mq_js_bundle.js` and before the `load("klondike-gui.wasm")` call. Only add `web/sapp_jsutils.js` if the bundle's built-in sapp_jsutils proves insufficient. Verify the page still loads.
- [x] 2.2 Copy the new JS into `dist/` from both `.github/workflows/deploy-pages.yml` and `tools/webcheck/build-dist.sh`; update the `CLAUDE.md` vendored-JS note. Verify `bash tools/webcheck/build-dist.sh` produces `dist/quad-storage.js`.

## 3. Record on win, counters, settings, display

- [x] 3.1 In `main.rs`, load `high_score: Option<HighScore>` and `counters: Counters` at startup, and add a per-game `new_high` flag; reset `new_high` in the existing `!is_won()` branch (new game). Verify `cargo build`.
- [x] 3.2 At the existing first-played-win seam: increment `counters.wins` and save; if `high_score.is_none_or(|h| final_score > h.score)`, build the record (`elapsed_secs`, timestamp `miniquad::date::now() as i64`), save, set `high_score` and `new_high = true`. Auto-solve is already excluded. Verify by winning: the record and wins count persist across a reload/restart and a lower later score doesn't overwrite the record.
- [x] 3.3 Increment `counters.new_games` and save on each fresh deal — the initial `Session::new` at startup and inside `new_game()` (route all new-game entry points through it so each counts once). Verify the count rises by one per new game and persists.
- [x] 3.4 Persist settings: load the settings blob at startup and apply it as the initial `Settings` (on native, an explicit launch arg still wins for that launch); save the blob whenever a value changes in `handle_settings`. Verify a changed setting is restored after relaunch — gui-shell scenario "Settings are restored on the next launch".
- [x] 3.5 Thread `high: Option<HighScore>` and `new_high` into `render::board`/`draw_win_banner`; show a "High score: N" line and a "New high score!" flourish when `new_high`. Verify the banner shows the stored high score and flags a new high — gui-shell scenarios "High score is shown on the banner" and "New high score is indicated".

## 4. Version bump & verification

- [x] 4.1 Bump the `Cargo.toml` patch version and rebuild so `Cargo.lock` updates. Verify the new version shows on the splash.
- [x] 4.2 Run `cargo build`, `cargo clippy`, and `cargo test`; verify all pass with no new warnings. Build the WASM target.
- [x] 4.3 Smoke-test the web build (`tools/webcheck` and/or a deploy): win a deal, confirm the high score shows and a new high is flagged, reload the page, and confirm the high score, counters, and settings persisted; verify no console/storage errors. On native, confirm files appear under `~/.config/klondike-solitaire/`.
