#!/usr/bin/env python3
"""Build the large-index "mobile" card set from the vector source deck.

Reads the SVG deck in ``assets/cards-svg/`` (named ``RANKSUIT.svg``, e.g. ``AS.svg``,
``10H.svg``) and writes recomposed PNGs to ``assets/cards-mobile/`` for use on small
/ touch screens, where a big rank + suit index (and a big body) stay readable even
when only a card's top strip shows in a tableau fan.

The source deck (750x1050 viewBox) is structurally regular: the first path is the
white card background, and the corner rank / corner pip / body are separate vector
elements. We use svgelements to get each element's exact bbox, classify rank / pip
/ body by position, then render each region CRISPLY by cloning the SVG, dropping
the white background, and windowing the root viewBox to that region (so cairosvg
renders only that region, at high resolution, with all transforms intact). Finally
we compose the three crisp region rasters into a mobile layout:

    big RANK + PIP on one top row, big BODY centered below.

Because every region is rendered from vectors at (or above) its final size, the
output stays crisp -- there is no upscaling of tiny raster corner glyphs.

Setup and usage (build-time tool only; the Rust game ships the generated PNGs):

    python3 -m venv .venv && . .venv/bin/activate
    pip install -r tools/requirements.txt
    python3 tools/build_mobile_cards.py            # assets/cards-svg -> assets/cards-mobile

cairosvg binds the system cairo library; on macOS: ``brew install cairo``. If cairo
is in a nonstandard prefix, set ``DYLD_FALLBACK_LIBRARY_PATH`` (Homebrew: /opt/homebrew/lib).
"""
import os
os.environ.setdefault("DYLD_FALLBACK_LIBRARY_PATH", "/opt/homebrew/lib")
import copy
import io
import sys

import cairosvg
from lxml import etree
from PIL import Image
from svgelements import SVG, Shape

SVG_NS = "http://www.w3.org/2000/svg"
NS = f"{{{SVG_NS}}}"
VBW, VBH = 750.0, 1050.0

CORNER = 0.25          # corner-zone extent (fraction of the viewBox)
REGION_PAD = 2.0       # viewBox padding (source units) so strokes aren't clipped
RENDER_LONG = 1200     # long-edge px when rendering a region (crispness)

# output layout (fractions of the output card)
OUT_W, OUT_H = 500, 700
MARGIN = 0.07
TOP_H = 0.30
GAP = 0.04
RANK_W_REL = 0.42
PIP_REL = 0.86
BODY_TOP = 0.40       # below the index row (margin + TOP_H) so the body never overlaps it
BODY_BOT = 0.98


def classify(svg_path):
    """Return (rank_bbox, pip_bbox, body_bbox) in source viewBox coords."""
    shapes = [e for e in SVG.parse(svg_path).elements() if isinstance(e, Shape)]
    corner, body = [], []
    for i, e in enumerate(shapes):
        bb = e.bbox()
        if bb is None or i == 0:      # i == 0 is the white background
            continue
        x0, y0, x1, y1 = bb
        cx, cy = (x0 + x1) / 2 / VBW, (y0 + y1) / 2 / VBH
        if cx < CORNER and cy < CORNER:
            corner.append((cy, bb))
        elif cx > 1 - CORNER and cy > 1 - CORNER:
            pass                      # bottom-right mirror -> discard
        else:
            body.append(bb)
    corner.sort(key=lambda t: t[0])   # topmost first = rank, then pip
    rank_bb = corner[0][1] if corner else None
    pip_bb = corner[1][1] if len(corner) > 1 else None
    body_bb = None
    if body:
        body_bb = (min(b[0] for b in body), min(b[1] for b in body),
                   max(b[2] for b in body), max(b[3] for b in body))
    return rank_bb, pip_bb, body_bb


def _strip_background(root):
    """Remove the opaque white card background so the render is transparent."""
    for child in root.iter(f"{NS}path"):
        style = (child.get("style") or "").lower().replace(" ", "")
        if "fill:white" in style or "fill:#fff" in style or "fill:#ffffff" in style:
            child.getparent().remove(child)
        break  # background is the first path


def render_region(src_path, bbox):
    """Crisp transparent raster of just `bbox`, via viewBox windowing."""
    tree = etree.parse(src_path)
    root = tree.getroot()
    _strip_background(root)
    x0, y0, x1, y1 = bbox
    x0 -= REGION_PAD; y0 -= REGION_PAD; x1 += REGION_PAD; y1 += REGION_PAD
    w, h = x1 - x0, y1 - y0
    root.set("viewBox", f"{x0} {y0} {w} {h}")
    for a in ("width", "height"):
        if a in root.attrib:
            del root.attrib[a]
    if w >= h:
        ow, oh = RENDER_LONG, max(1, round(RENDER_LONG * h / w))
    else:
        ow, oh = max(1, round(RENDER_LONG * w / h)), RENDER_LONG
    png = cairosvg.svg2png(bytestring=etree.tostring(root),
                           output_width=ow, output_height=oh)
    im = Image.open(io.BytesIO(png)).convert("RGBA")
    bb = im.getchannel("A").point(lambda a: 255 if a > 8 else 0).getbbox()
    return im.crop(bb) if bb else im


def paste_fit(canvas, piece, box, halign="center", valign="center"):
    if piece is None:
        return
    dx0, dy0, dx1, dy1 = box
    dw, dh = dx1 - dx0, dy1 - dy0
    pw, ph = piece.size
    s = min(dw / pw, dh / ph)
    nw, nh = max(1, round(pw * s)), max(1, round(ph * s))
    piece = piece.resize((nw, nh), Image.LANCZOS)
    x = dx0 if halign == "left" else (dx1 - nw if halign == "right" else dx0 + (dw - nw) // 2)
    y = dy0 if valign == "top" else (dy1 - nh if valign == "bottom" else dy0 + (dh - nh) // 2)
    canvas.alpha_composite(piece, (x, y))


def build_card(src_path, scale=2):
    rank_bb, pip_bb, body_bb = classify(src_path)
    rank = render_region(src_path, rank_bb) if rank_bb else None
    pip = render_region(src_path, pip_bb) if pip_bb else None
    body = render_region(src_path, body_bb) if body_bb else None

    W, H = OUT_W * scale, OUT_H * scale
    canvas = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    m = round(MARGIN * W)
    top_h = round(TOP_H * H)
    rank_w = round((W - 2 * m) * RANK_W_REL)

    paste_fit(canvas, rank, (m, m, m + rank_w, m + top_h), halign="left")
    ph = round(top_h * PIP_REL)
    pad = (top_h - ph) // 2
    px = m + rank_w + round(GAP * W)
    paste_fit(canvas, pip, (px, m + pad, px + ph, m + pad + ph), halign="left")
    paste_fit(canvas, body, (m, round(BODY_TOP * H), W - m, round(BODY_BOT * H)))
    return canvas


def main():
    src_dir = sys.argv[1] if len(sys.argv) > 1 else "assets/cards-svg"
    out_dir = sys.argv[2] if len(sys.argv) > 2 else "assets/cards-mobile"
    os.makedirs(out_dir, exist_ok=True)
    names = sys.argv[3:] or sorted(f[:-4] for f in os.listdir(src_dir) if f.endswith(".svg"))
    for n in names:
        build_card(os.path.join(src_dir, f"{n}.svg")).save(os.path.join(out_dir, f"{n}.png"))
    print(f"built {len(names)} cards -> {out_dir}")


if __name__ == "__main__":
    main()
