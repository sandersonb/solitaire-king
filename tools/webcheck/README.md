# webcheck — Playwright harness for the web (WASM) build

Drives Chromium against the deployed or a locally-built page and captures how it
loads and renders: a **video**, a burst of **interval screenshots**, the browser
**console**, and a **network waterfall**. It prints a per-frame byte-size
timeline, which is a cheap flicker detector — a blank/black frame compresses to a
tiny PNG, a full frame is large, so sudden dips reveal dropped/black frames.

Built to diagnose web-only issues that never show up on the native build or a
fast desktop GPU (e.g. the iOS Safari load-screen flicker).

## One-time setup

```sh
cd tools/webcheck
npm install
npx playwright install chromium
```

## Capture a local build (recommended for iterating on a fix)

From the repo root, assemble a `dist/` (release WASM + assets, mirroring the
Pages deploy), then capture it — the script serves the folder itself:

```sh
bash tools/webcheck/build-dist.sh          # -> tools/webcheck/dist/
cd tools/webcheck
DIR=./dist npm run capture
```

Re-run `build-dist.sh` and `DIR=./dist npm run capture` after each code change.

## Capture the live site

```sh
cd tools/webcheck
npm run capture -- https://sandersonb.github.io/solitaire-king/
```

## Options (env vars)

| var      | default | meaning                                                        |
|----------|---------|----------------------------------------------------------------|
| `DIR`    | —       | serve this folder locally and capture it (omit to use the URL) |
| `HEADED` | `0`     | `1` opens a real window (real-GPU WebGL — see note below)       |
| `W` `H`  | 900 1400| viewport size                                                  |
| `SCALE`  | 2       | device pixel ratio (use 3 to mimic a modern iPhone)            |
| `MS`     | 8000    | total capture duration (ms)                                    |
| `STEP`   | 90      | screenshot interval (ms) — lower to catch faster flicker       |
| `OUT`    | `out`   | output directory                                               |
| `PORT`   | 8123    | port for the built-in static server (with `DIR`)               |

Example — iPhone-ish, headed, dense sampling:

```sh
DIR=./dist HEADED=1 W=390 H=844 SCALE=3 STEP=40 npm run capture
```

## Reading the output

- `out/shots/NNN_Tms.png` — screenshots; open the small ones to see blank/black
  frames.
- The printed **frame-size timeline** — watch for dips to tiny sizes (flicker).
- `out/video/*.webm` — the recording (the only faithful record of a fast flicker;
  extract frames with `ffmpeg -i out/video/*.webm out/frames/%04d.png`).
- `out/console.txt`, `out/net.json` — console logs and request timings.

## Notes

- **Headless renders WebGL in software (SwiftShader).** GPU/compositor-timing
  bugs may not reproduce headless; use `HEADED=1` to render on the real GPU.
- `dist/`, `out/`, and `node_modules/` are git-ignored.
