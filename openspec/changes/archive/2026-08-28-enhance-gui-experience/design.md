## Context

See `proposal.md` — Why. Relevant current state:

- The GUI (`src/gui/`) is a macroquad app targeting native and `wasm32` (WebGL2).
  Text is drawn with macroquad's built-in bitmap font via `draw_text`. Cards are
  PNG sprites loaded from `assets/cards/` with a procedural fallback.
- Input (`src/gui/input.rs`, `main.rs`) is a click-select → click-target model
  with a persistent `Option<Source>` selection and a yellow highlight. Moves
  resolve through the pure `resolve`/`auto_target` functions over `legal_moves`.
- Layout (`src/gui/layout.rs`) is a single responsive layout: 7 columns across,
  cards sized by the binding of width/height budgets.
- Seeds are `u64` throughout: `GameState::new_with_seed`, `--seed <u64>` in
  `src/main.rs` (clap), and displayed raw in CLI render, solver report, and GUI
  status. The deal is fully determined by the `u64` (SplitMix64 shuffle) — this
  change must not alter that mapping.
- Assets are served under `dist/assets/` on web (`set_pc_assets_folder("assets")`);
  the deploy workflow copies `assets/` and vendors `web/mq_js_bundle.js`.

Constraints: the library target is `std`-only and compiles for `wasm32`; the CLI
deps (clap, crossterm) are native-only. Anything shared by CLI and GUI (seed
encoding) must live in the library and stay `std`-only.

## Goals / Non-Goals

**Goals:**
- One seed-encoding module in the library, reused by CLI and GUI, with raw-`u64`
  back-compat and no change to the deal mapping.
- Crisp text at high DPI via a bundled font, with graceful fallback.
- Playable by touch on a phone: responsive mobile layout, on-screen Undo/New,
  touch drag, and a mobile card image set.
- Drag-and-drop replacing click-select, with drop zones and a reusable animation
  subsystem that also supports future automated move playback.

**Non-Goals:**
- No solver integration in the GUI yet (only the animation hooks that make it
  cheap later). No "gesture" vocabulary is defined here beyond leaving room for it.
- No change to rules, scoring, RNG, deal order, or the solver.
- No new runtime crate dependencies (font/proquint done in-crate; macroquad
  already provides TTF loading and touch input).
- No redo button (per product decision); redo stays keyboard-only.

## Decisions

### D1: Seed encoding = proquint, in the library
New module `src/model/seed.rs`, re-exported as `klondike::seed::{encode, decode}`
(`encode(u64) -> String`, `decode(&str) -> Option<u64>`).
- Proquint packs 16 bits per 5-char consonant-vowel "quint"; a `u64` → four quints
  joined by `-` (e.g. `lusab-babad-gutih-tugad`). Fixed alphabet, pure arithmetic,
  no data files, trivially `wasm32`-safe.
- `decode` is case-insensitive, strips `-`/whitespace, and if the input isn't a
  valid quint sequence it falls back to `str::parse::<u64>()` so every legacy raw
  seed still resolves; otherwise returns `None`.
- Unit tests: round-trip over many `u64` (incl. 0 and `u64::MAX`), raw-`u64`
  fallback, and rejection of garbage.
- *Alternatives:* Crockford base32 (compact but not pronounceable), BIP39 words
  (most memorable but bundles a 2048-word list and is long). Proquint chosen for
  memorability with zero data and short length. (Decided with the user.)

### D2: CLI seed plumbing
`--seed` type changes from `u64` to `String`; parse with `seed::decode` after clap.
An undecodable value prints an error + usage and exits (matches the existing
invalid-flag behavior). Display sites (`cli/render.rs` status, `cli/solve.rs`
report, `main.rs` quit summary) print `seed::encode(seed)`.
- *Alternative:* a custom clap value parser returning `u64` directly. Deferred —
  parsing after `Args::parse()` keeps the decode error message uniform with the
  existing `build_config` error path.

### D3: Font as a bundled asset
Bundle one OFL sans (a clean, legible face with good tabular numerals — e.g.
Atkinson Hyperlegible / Inter) at `assets/fonts/ui.ttf`. Load via
`load_ttf_font` in `assets.rs` into `Assets { font: Option<Font> }`. All GUI text
goes through a helper `text(&Assets, ...)` that uses `draw_text_ex` with the font
when present and `draw_text` otherwise. `measure_text` takes the same font so
centering stays correct.
- *Trade-off:* adds ~a few hundred KB to the wasm payload/asset set; acceptable and
  cached. Fallback to built-in font preserves the "always playable" invariant.

### D4: Responsive layout + control bar
`Layout::compute` gains the viewport aspect/size (and a touch flag) and picks a
profile: **desktop** (current behavior) or **mobile/portrait** (reserve a bottom
band for the control bar; size cards from the remaining area; keep 7 columns).
Add `Layout::buttons: Vec<(ButtonId, Rect)>` and `Layout::drop_zones()` deriving a
drop-zone `Rect` per pile (foundations, tableau columns, and — for a return —
the origin). Drop zones are generous (pile rect padded) so release need not be
precise. Touch detection: `miniquad`/macroquad touch presence or a narrow-aspect
heuristic; the control bar also renders on desktop (harmless, discoverable).

### D5: Mobile card art with fallback chain
Load an optional second set from `assets/cards-mobile/` into
`Assets { cards_mobile: HashMap<...> }`. `draw_card` resolves in order: mobile set
(when on a mobile profile and present) → desktop set → procedural. The back and
placeholders are shared. Deploy workflow ships `cards-mobile/` alongside `cards/`.
- *Trade-off:* extra assets/download on mobile (accepted by the user) in exchange
  for legibility; the fallback means a missing set never breaks play.

### D6: Drag-and-drop input model
Replace `Option<Source>` selection with a `Drag` state:
`{ source: Source, run: Vec<Card>, grab_offset, pointer, origin_rects }`.
- **Press** (mouse or first touch) hit-tests a source via existing `source_of`;
  stock press still deals/recycles; empty press is a no-op.
- **Move** updates the pointer; the run renders following it. On touch the run is
  lifted above the finger and scaled up so it isn't occluded (this is the mobile
  readability aid, complementing D5).
- **Release** picks the nearest drop zone within threshold, then reuses the pure
  `resolve(state, source, pile)` to get a legal `Move`. Legal → apply + enqueue a
  snap animation from release point to the pile's resting rect. Illegal/none →
  enqueue a return animation to origin and show the existing rejection message.
- Double-click/tap and Enter still call `auto_target`. Keyboard commands (native)
  are unchanged. The yellow selection highlight is removed.
- Unifying mouse+touch: a small `Pointer` abstraction reads
  `mouse_position()`/button state or `touches()` so the resolution logic is
  input-source-agnostic — this is also the seam future "gestures" plug into.

### D7: Animation subsystem (`src/gui/anim.rs`)
A `CardAnim { card(s), from: Vec2, to: Vec2, t0, dur, ease }` list ticked each
frame by elapsed time; the renderer draws in-flight cards at the interpolated
position on top of the board. Key rule: **state changes first, animation is
cosmetic** — `session.apply(mv)` runs on release, then the anim plays; input and
scoring never wait on it. An `enqueue_moves(&[Move])` entry point applies+animates
a sequence in order, giving future auto-solve playback for free.
- *Alternative:* animate before committing state (drag the model). Rejected — it
  complicates undo/legality and blocks input; cosmetic-only is simpler and matches
  the spec ("state already reflects the move").

## Risks / Trade-offs

- **Touch/mouse event duplication on web** (browsers synthesize mouse events from
  touch) → route through the single `Pointer` abstraction and prefer `touches()`
  when any touch is active, so a tap isn't handled twice.
- **Snap animation vs. a rapid next input** → animations are cosmetic and
  interruptible; a new press immediately supersedes an in-flight cosmetic anim for
  that card. State is always already correct.
- **Wasm payload growth** (font + mobile card set) → both are optional assets with
  fallbacks; ship compressed; only the mobile set is fetched on mobile profiles.
- **Drop-zone overlap ambiguity** (adjacent columns) → pick the zone by smallest
  center distance among zones containing/nearest the release point; ties break to
  the pile under the pointer.
- **Proquint edge cases** (0, `u64::MAX`, odd casing, stray separators) → covered
  by round-trip and fallback unit tests before any UI wiring.
- **Deploy drift** (new asset dirs not shipped) → update the Pages workflow's copy
  step in the same change; CLAUDE.md already flags the assets-path coupling.

## Migration Plan

1. Land `seed` module + tests (pure, no UI). 2. Wire CLI display + `--seed`
parsing. 3. Bundle font + asset loading + `text()` helper. 4. Responsive layout +
control bar + touch controls. 5. Mobile card set + fallback. 6. Drag-drop +
animation subsystem (remove selection highlight last). 7. Update deploy workflow
and CLAUDE.md for new assets. Rollback is per-step and low-risk: the seed layer is
presentation-only, and every new asset has a fallback, so partial reverts leave a
playable game.
