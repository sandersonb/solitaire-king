## Context

See `proposal.md` — Why. Measured deployed payload:

- ~107 startup fetches: 52 `cards/` faces (222×323, ~1.4 MB), 52 `cards-mobile/`
  faces (600×840, 4.1 MB), `cards/back.png` (**796 KB @ 1280×1792**),
  `king-logo.png` (**1.5 MB @ 766×975, fully opaque**), `fonts/ui.ttf` (~56 KB).
- `assets.rs` loads every face into its own `Texture2D`; `face(rank, suit, mobile)`
  returns that per-card texture; `render.rs` draws it with `draw_texture_ex`. Both
  decks are fetched for every client.
- `tools/build_mobile_cards.py` already renders the SVG deck with cairosvg + PIL, so
  atlas packing and resizing fit its pipeline.
- macroquad's `Texture2D::from_file_with_format(bytes, None)` autodetects PNG **and
  JPEG**, and `DrawTextureParams` has a `source: Option<Rect>` for sub-rect draws.

## Goals / Non-Goals

**Goals:**
- Collapse ~104 face requests into a handful (atlases) — the main mobile win.
- Materially cut transfer bytes by right-sizing the clearly-oversized art (back,
  logo→JPEG) and lossless-optimizing PNGs, without visibly degrading faces.
- Preserve appearance and the procedural fallback; native behaves the same.

**Non-Goals:**
- No redesign of the card art; no runtime SVG; no wasm-size work.
- Not aggressively downscaling faces (user wants high quality — mild only).
- Minimal new deps. (Implementation note: macroquad's bundled `image` enables only
  `png`+`tga`, so the opaque JPEG logo needs the `image` crate's `jpeg` feature —
  added as a direct dep so Cargo feature-unification lets macroquad decode it. Adds
  ~150 KB to the wasm, far less than the JPEG saving.)

## Decisions

### 1. Per-deck texture atlases (regular grid)
Generate one-or-more atlas PNGs per deck by packing faces into a **13-rank × 4-suit
regular grid**, so a card's source rect is computed from `(rank_index, suit_index)`
— no mapping file. A small transparent **gutter** (a few px) separates cells, and
the renderer samples the exact cell rect, so linear filtering can't bleed a
neighbor in.

*Texture-size limit:* WebGL2 guarantees only 2048² but virtually all target devices
(the user's iPhone/iPad) support ≥4096. Keep every atlas ≤ ~4096 on its long edge;
if a deck at its chosen cell size won't fit in one atlas, split into the fewest
atlases that do (e.g. the mobile deck across 2–3 atlases). Desktop faces (222×323)
fit one atlas (~2886×1292) at native size.

*Cell resolution (the "sweet spot"):* desktop faces stay **native**; mobile faces
take at most a **mild** reduction if needed to bound atlas count — quality first.
The atlas mainly saves *requests*, not bytes.

### 2. Right-size the oversized art (byte win)
- `back.png` 1280×1792 → ~512×717 (still 2–3× its ~150 px draw size; crisp on
  Retina): ~-0.7 MB.
- `king-logo.png` → **`king-logo.jpg`** (opaque, so no alpha lost), modest size and
  quality ~85–90: ~1.5 MB → ~0.1–0.2 MB.
- Lossless PNG optimization (e.g. `oxipng`) on the atlases and back.
- These are the transfer sweet spot: huge savings where the image was far oversized,
  none taken from the faces.

### 3. Runtime: atlas sampling behind `assets.rs`
`Assets` holds the atlas texture(s) instead of per-card maps.
`face(rank, suit, mobile_deck)` returns `Option<(&Texture2D, Rect)>` (atlas + pixel
source rect). `draw_card` and the celebration's `draw_rot_card` draw with
`source: Some(rect)` (add `source` to the `tex_params*` helpers). The procedural
fallback path is unchanged. The loader's job list shrinks from ~107 to: the
atlas(es) + back + logo + font.

### 4. Deck loading
Load the atlas(es) for the viewport's deck first; the other deck's atlas MAY be
loaded eagerly too (few requests) or lazily on first use if the deck override
selects it — chosen at apply for the best mobile first-paint. This coordinates with
the (separate) persisted deck setting; if that change isn't applied, `mobile_deck`
still drives selection.

## Risks / Trade-offs

- [GPU max texture size on a low-end device] → atlases kept ≤ ~4096, split as
  needed; a device below that (rare for the target) would fail to load one big
  texture, hence the cap.
- [Atlas edge bleeding under linear filtering] → transparent gutter between cells +
  exact source rects (Decision 1).
- [Build step grows] → atlas/resize/optimize run in the existing Python tool;
  document regeneration in `CLAUDE.md` (like the current mobile-set step).
- [Coupling with `render.rs` and the deploy workflow, also touched by
  add-high-score-persistence] → independent edits; apply in either order and
  reconcile the small overlaps.
- [Interaction with the persisted deck override] → see Decision 4; `store`/settings
  are not required for this change to stand alone.

## Open Questions

None blocking. Exact cell resolutions, atlas count, JPEG quality, and back size are
tuned by eye during apply against the transfer/quality trade — they don't change the
spec or task breakdown.
