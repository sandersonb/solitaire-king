## Why

The browser build downloads **~107 separate files** at startup (52 desktop + 52
mobile card faces, the card back, logo, and font) totaling ~7.9 MB of art. On
mobile, the per-request latency of 100+ round trips dominates the load, and several
images are far larger than they're ever drawn (`back.png` is 796 KB at 1280×1792
for a card shown ~150 px wide; `king-logo.png` is 1.5 MB). This makes the first
paint slow, especially on phones.

## What Changes

- **Pack card faces into texture atlases.** Each deck's 52 faces SHALL be combined
  into a small number of atlas images (a regular rank×suit grid, so each card's
  sub-region is computed — no mapping file), collapsing ~104 face requests to a
  handful. The renderer SHALL draw each card from its sub-region of the atlas.
- **Right-size the oversized art (gently).** `back.png` SHALL be reduced from
  1280×1792 to near its display size; `king-logo.png` SHALL be delivered as a
  **JPEG** (it is fully opaque) at a modest size. Card **faces** keep their quality
  — desktop faces stay native, the mobile set takes only a mild reduction — and all
  PNGs are losslessly optimized. The aim is the transfer sweet spot, not aggressive
  downscaling.
- **Preserve behavior.** Cards look the same and the procedural fallback (when art
  is absent) is unchanged; this is a delivery/perf change only.
- **Version bump.** Increment the crate patch version.

## Capabilities

### New Capabilities
<!-- None: refines how the existing browser build delivers assets. -->

### Modified Capabilities
- `gui-distribution`: add an efficient-startup-asset-delivery requirement — the
  browser build combines card art into atlases and right-sizes/compresses oversized
  images to minimize startup requests and transfer size (especially on mobile),
  while preserving the procedural fallback.

## Impact

- `tools/build_mobile_cards.py` (and/or a companion script) — also emit per-deck
  **atlas** PNGs (desktop + mobile) sized within a safe GPU texture limit, plus the
  right-sized `back` and the JPEG logo; run lossless PNG optimization.
- `assets/` — replace the per-card PNG sets with the generated atlases + right-sized
  `back` + `king-logo.jpg`; the deploy prunes build-only sources as today.
- `src/gui/assets.rs` — load atlas texture(s); `face(..)` returns the atlas texture
  plus the card's source rect. `src/gui/render.rs` — draw cards (and the celebration
  cards) from a source sub-rect. Loader fetches a handful of files, not ~107.
- `.github/workflows/deploy-pages.yml` and `tools/webcheck/build-dist.sh` — ship the
  new asset set; `CLAUDE.md` asset notes updated.
- No changes to the domain, rules, solver, or CLI. Native behaves the same.
