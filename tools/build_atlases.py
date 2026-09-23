#!/usr/bin/env python3
"""Pack the per-card PNG decks into texture atlases and right-size the oversized
art, for a fast-loading web build.

Runtime assets written to ``assets/`` (shipped):
  - ``cards-atlas.png``         desktop deck, 13x4 grid at native cell size
  - ``cards-mobile-atlas.png``  mobile deck, 9-col grid, mildly downscaled cell
  - ``back.png``                the card back, right-sized from the huge original
  - ``king-logo.jpg``           the splash/banner logo as opaque JPEG

Build-only sources (kept in the repo, pruned from the deploy like ``cards-svg/``):
  - ``cards/`` and ``cards-mobile/`` per-card PNGs (read here)
  - ``king-logo.png`` original, ``cards/back.png`` original

The atlas grid convention is mirrored in ``src/gui/assets.rs`` (GRID must match):
  card index i = suit_index*13 + rank_index   (suits C,D,H,S; ranks A..K)
  col = i % cols,  row = i // cols
  a cell's top-left in the atlas is (GUTTER + col*(cw+GUTTER), GUTTER + row*(ch+GUTTER))
A transparent GUTTER around every cell keeps linear filtering from bleeding a
neighbour in.

Setup / usage (build-time only; the game ships the generated files):

    python3 -m venv .venv && . .venv/bin/activate
    pip install Pillow
    python3 tools/build_atlases.py
"""
import os
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "assets")

RANKS = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"]
SUITS = ["C", "D", "H", "S"]  # Suit::ALL order

GUTTER = 4  # transparent px around each cell; must match assets.rs

# (source dir, output file, columns, cell width, cell height). Cells are resized
# to (cw, ch) so the grid is uniform. Desktop keeps native size; mobile is mildly
# reduced (0.75x) so all 52 fit one atlas under the 4096 texture-size cap.
DECKS = [
    ("cards", "cards-atlas.png", 13, 222, 323),
    ("cards-mobile", "cards-mobile-atlas.png", 9, 450, 630),
]


def build_atlas(src_dir, out_name, cols, cw, ch):
    rows = (52 + cols - 1) // cols
    width = GUTTER + cols * (cw + GUTTER)
    height = GUTTER + rows * (ch + GUTTER)
    atlas = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    for si, suit in enumerate(SUITS):
        for ri, rank in enumerate(RANKS):
            i = si * 13 + ri
            col, row = i % cols, i // cols
            path = os.path.join(ASSETS, src_dir, f"{rank}{suit}.png")
            face = Image.open(path).convert("RGBA")
            if face.size != (cw, ch):
                face = face.resize((cw, ch), Image.LANCZOS)
            x = GUTTER + col * (cw + GUTTER)
            y = GUTTER + row * (ch + GUTTER)
            atlas.paste(face, (x, y))
    out = os.path.join(ASSETS, out_name)
    atlas.save(out, optimize=True)
    print(f"  {out_name}: {width}x{height}  {os.path.getsize(out)//1024} KiB")


def main():
    print("atlases:")
    for src, out, cols, cw, ch in DECKS:
        build_atlas(src, out, cols, cw, ch)

    # Right-size the card back (drawn ~150px wide; original is 1280x1792).
    back = Image.open(os.path.join(ASSETS, "cards", "back.png")).convert("RGBA")
    back = back.resize((512, 717), Image.LANCZOS)
    back_out = os.path.join(ASSETS, "back.png")
    back.save(back_out, optimize=True)
    print(f"back.png: {back.size[0]}x{back.size[1]}  {os.path.getsize(back_out)//1024} KiB")

    # Logo is fully opaque -> JPEG at native size, big saving vs the 1.5 MB PNG.
    logo = Image.open(os.path.join(ASSETS, "king-logo.png")).convert("RGB")
    logo_out = os.path.join(ASSETS, "king-logo.jpg")
    logo.save(logo_out, quality=88, optimize=True, progressive=True)
    print(f"king-logo.jpg: {logo.size[0]}x{logo.size[1]}  {os.path.getsize(logo_out)//1024} KiB")


if __name__ == "__main__":
    main()
