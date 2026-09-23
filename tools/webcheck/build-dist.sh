#!/usr/bin/env bash
# Assemble the web build into tools/webcheck/dist/, mirroring the GitHub Pages
# deploy workflow, so `capture.mjs` can serve and inspect it locally.
set -euo pipefail

ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$ROOT"

cargo build --release --target wasm32-unknown-unknown --bin klondike-gui

DIST="tools/webcheck/dist"
rm -rf "$DIST"
mkdir -p "$DIST"
cp web/index.html web/mq_js_bundle.js web/sapp_jsutils.js web/quad-storage.js "$DIST/"
cp target/wasm32-unknown-unknown/release/klondike-gui.wasm "$DIST/"
cp -r assets "$DIST/assets"
# Prune build-time-only sources: the vector deck, the per-card PNG decks (packed
# into cards-atlas.png / cards-mobile-atlas.png by tools/build_atlases.py), and the
# original logo (shipped as king-logo.jpg). See the deploy workflow.
rm -rf "$DIST/assets/cards-svg" "$DIST/assets/cards" "$DIST/assets/cards-mobile" \
       "$DIST/assets/king-logo.png"

echo "dist assembled at $DIST"
