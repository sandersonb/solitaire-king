// Capture the web build's load/render behavior with Playwright: a video, a burst
// of interval screenshots, the browser console, and a network waterfall. Prints a
// per-frame byte-size timeline — a cheap way to spot black/flicker frames, since a
// blank/dark frame compresses to a tiny PNG while a full frame is large.
//
// Usage:
//   DIR=./dist node capture.mjs           # serve ./dist locally and capture
//   node capture.mjs http://host/page     # capture an already-served URL
// Env: HEADED=1 (real-GPU window), W, H, SCALE, MS (total capture), STEP (ms).
//
// Why HEADED matters: headless Chromium renders WebGL via software (SwiftShader),
// so GPU/compositor-timing bugs may not reproduce. Use HEADED=1 for those.

import { chromium } from 'playwright';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

const DIR = process.env.DIR || null;
const PORT = Number(process.env.PORT || 8123);
const HEADED = process.env.HEADED === '1';
const W = Number(process.env.W || 900);
const H = Number(process.env.H || 1400);
const SCALE = Number(process.env.SCALE || 2);
const MS = Number(process.env.MS || 8000);
const STEP = Number(process.env.STEP || 90);
const OUT = process.env.OUT || 'out';

const MIME = {
  '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm',
  '.png': 'image/png', '.ttf': 'font/ttf', '.json': 'application/json',
};

// Optionally serve a directory so the harness is one command.
let server = null;
let URL = process.argv[2] || `http://localhost:${PORT}/index.html`;
if (DIR) {
  const root = path.resolve(DIR);
  server = http.createServer((req, res) => {
    const rel = decodeURIComponent(req.url.split('?')[0]).replace(/^\/+/, '');
    const file = path.join(root, rel || 'index.html');
    fs.readFile(file, (err, buf) => {
      if (err) { res.writeHead(404); res.end('not found'); return; }
      res.writeHead(200, { 'content-type': MIME[path.extname(file)] || 'application/octet-stream' });
      res.end(buf);
    });
  }).listen(PORT);
  URL = `http://localhost:${PORT}/index.html`;
}

fs.rmSync(OUT, { recursive: true, force: true });
fs.mkdirSync(OUT + '/shots', { recursive: true });

const browser = await chromium.launch({
  headless: !HEADED,
  args: ['--use-gl=angle', '--use-angle=metal', '--ignore-gpu-blocklist', '--enable-gpu'],
});
const context = await browser.newContext({
  viewport: { width: W, height: H },
  deviceScaleFactor: SCALE,
  recordVideo: { dir: OUT + '/video', size: { width: W, height: H } },
});
const page = await context.newPage();

const consoleLines = [];
page.on('console', (m) => consoleLines.push(`[${m.type()}] ${m.text()}`));
page.on('pageerror', (e) => consoleLines.push(`[pageerror] ${e.message}`));

const net = [];
const t0 = Date.now();
page.on('requestfinished', async (req) => {
  const resp = await req.response().catch(() => null);
  net.push({ t: Date.now() - t0, url: req.url().split('/').pop(), status: resp?.status() });
});

const nav = page.goto(URL, { waitUntil: 'commit' }).catch(() => {});
const shots = [];
const start = Date.now();
while (Date.now() - start < MS) {
  const ts = Date.now() - start;
  try {
    const buf = await page.screenshot({ timeout: 2000 });
    fs.writeFileSync(`${OUT}/shots/${String(shots.length).padStart(3, '0')}_${ts}ms.png`, buf);
    shots.push({ ts, bytes: buf.length });
  } catch (e) {
    shots.push({ ts, err: String(e).slice(0, 60) });
  }
  await page.waitForTimeout(STEP);
}
await nav;

fs.writeFileSync(OUT + '/console.txt', consoleLines.join('\n'));
fs.writeFileSync(OUT + '/net.json', JSON.stringify(net, null, 2));
fs.writeFileSync(OUT + '/shots.json', JSON.stringify(shots, null, 2));
const vids = fs.readdirSync(OUT + '/video').map((v) => `${OUT}/video/${v}`);

await context.close(); // finalizes the video
await browser.close();
if (server) server.close();

// --- report ---------------------------------------------------------------
const sized = shots.filter((s) => s.bytes);
const max = Math.max(...sized.map((s) => s.bytes));
console.log('\nframe-size timeline (tiny frames are likely blank/black):');
for (const s of shots) {
  const b = s.bytes || 0;
  console.log(`  ${String(s.ts).padStart(5)}ms ${String(b).padStart(7)}  ${'#'.repeat(Math.round(b / (max / 50)))}`);
}
// Flag frames far smaller than the max as suspicious (possible flicker).
const suspicious = sized.filter((s) => s.bytes < max * 0.15);
console.log(`\nsuspicious dark/blank frames (< 15% of max size): ${suspicious.length}`);
console.log('network:', net.slice(0, 8).map((n) => `+${n.t}ms ${n.url}`).join('  '));
console.log('console lines:', consoleLines.length, '(see', OUT + '/console.txt)');
console.log('video:', vids.join(', '));
console.log('screenshots:', OUT + '/shots/');
