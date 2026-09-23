## 1. Asset generation (build tooling)

- [x] 1.1 Extend `tools/build_mobile_cards.py` (or add a companion `tools/build_atlases.py`) to emit per-deck atlas PNG(s): faces packed in a 13-rank × 4-suit grid with a few-px transparent gutter, each atlas kept ≤ ~4096 on its long edge (split into multiple atlases if needed). Desktop faces at native 222×323; mobile faces native or mildly reduced. Emit a documented, deterministic cell order so source rects are computable. Verify the atlases open and are within the size cap.
- [x] 1.2 Right-size the oversized art: generate `back.png` at ~512×717, and `king-logo.jpg` (JPEG q≈85–90, opaque source) at a modest size; run `oxipng` (or equivalent) on the atlases and back. Verify the new files exist and are materially smaller (target: logo ~1.5 MB → ≤0.2 MB, back ~0.8 MB → ≤0.15 MB).
- [x] 1.3 Update `assets/` to the generated set (atlases + right-sized back + `king-logo.jpg`), and document regeneration + the atlas grid convention in `CLAUDE.md`. Verify the runtime asset dir no longer contains 100+ per-card PNGs.

## 2. Runtime loading + rendering

- [x] 2.1 In `src/gui/assets.rs`, load the atlas texture(s), the right-sized back, and `king-logo.jpg`; change `face(rank, suit, mobile_deck)` to return `Option<(&Texture2D, Rect)>` (atlas + pixel source rect computed from rank/suit). Shrink the loader's job list to the atlas(es) + back + logo + font. Verify `cargo build` (native + wasm).
- [x] 2.2 In `src/gui/render.rs`, add a `source: Option<Rect>` to the `tex_params`/`tex_params_rot` helpers and draw `draw_card` and the celebration `draw_rot_card` from the card's atlas sub-rect; keep the procedural fallback unchanged. Verify cards render correctly with no edge bleeding between neighbors (gutter + exact source rect).
- [x] 2.3 Decide deck-load strategy (eager both atlases vs eager-viewport + lazy-other) and implement; verify the mobile viewport paints its deck without fetching the unused deck up front (or that both are just a couple of requests). Verify `cargo build`.

## 3. Distribution plumbing

- [x] 3.1 Update `.github/workflows/deploy-pages.yml` and `tools/webcheck/build-dist.sh` to ship the new asset set (atlases, `back`, `king-logo.jpg`) and prune build-only sources. Verify `bash tools/webcheck/build-dist.sh` assembles a `dist/` with the atlases and JPEG logo and no per-card PNGs.

## 4. Version bump & verification

- [x] 4.1 Bump the `Cargo.toml` patch version and rebuild so `Cargo.lock` updates. Verify the new version shows on the splash.
- [x] 4.2 Run `cargo build`, `cargo clippy`, and `cargo test`; verify all pass with no new warnings. Build the WASM target.
- [x] 4.3 Capture with `tools/webcheck` before/after: confirm the startup **request count** drops from ~107 to a handful and total transfer is materially lower (network waterfall), the board and splash render correctly (cards, back, logo), and there is no visual regression — gui-distribution scenarios "Few requests at startup", "Oversized art is right-sized", and "Appearance and fallback preserved".
