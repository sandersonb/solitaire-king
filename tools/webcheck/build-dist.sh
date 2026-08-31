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
cp web/index.html web/mq_js_bundle.js "$DIST/"
cp target/wasm32-unknown-unknown/release/klondike-gui.wasm "$DIST/"
cp -r assets "$DIST/assets"
rm -rf "$DIST/assets/cards-svg"   # build-time only, not served (see deploy workflow)

echo "dist assembled at $DIST"
