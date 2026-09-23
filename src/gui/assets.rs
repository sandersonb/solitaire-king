//! Asset loading. Raw file bytes are fetched in a background coroutine while the
//! main loop draws the loading screen every frame and decodes a few blobs into
//! textures per frame. This matters on web: decoding in the async pre-loop parked
//! on each `fetch`, so RAF ticks passed with nothing drawn and the canvas showed
//! black between frames (a flicker on real GPUs). Driving the load from the main
//! loop means every frame presents a drawn frame. Missing assets are tolerated —
//! the renderer falls back to procedural cards.
//!
//! Card faces ship as one texture **atlas** per deck (a rank×suit grid) rather
//! than one file per card, so the web build makes a handful of requests instead of
//! ~100. `Atlas::source` maps `(rank, suit)` to the card's sub-rect; the grid
//! convention here must match `tools/build_atlases.py`.

use std::sync::{Arc, Mutex};

use macroquad::experimental::coroutines::{start_coroutine, Coroutine};
use macroquad::prelude::*;

use klondike::{Rank, Suit};

/// Transparent padding around each atlas cell (must match `tools/build_atlases.py`).
const GUTTER: f32 = 4.0;

/// A packed card-face atlas plus the grid geometry to address it.
pub struct Atlas {
    tex: Texture2D,
    /// Columns in the grid; `col = i % cols`, `row = i / cols`.
    cols: usize,
    /// Cell size in atlas pixels.
    cell_w: f32,
    cell_h: f32,
}

impl Atlas {
    fn new(tex: Texture2D, cols: usize, cell_w: f32, cell_h: f32) -> Atlas {
        tex.set_filter(FilterMode::Linear);
        Atlas {
            tex,
            cols,
            cell_w,
            cell_h,
        }
    }

    /// Source rect (in atlas pixels) for a card. `i = suit_index*13 + rank_index`.
    fn source(&self, rank: Rank, suit: Suit) -> Rect {
        let i = suit_index(suit) * 13 + (rank.value() as usize - 1);
        let col = (i % self.cols) as f32;
        let row = (i / self.cols) as f32;
        Rect::new(
            GUTTER + col * (self.cell_w + GUTTER),
            GUTTER + row * (self.cell_h + GUTTER),
            self.cell_w,
            self.cell_h,
        )
    }
}

/// Grid parameters per deck: (atlas filename, columns, cell_w, cell_h). Must match
/// `tools/build_atlases.py`.
const DESKTOP_ATLAS: (&str, usize, f32, f32) = ("cards-atlas.png", 13, 222.0, 323.0);
const MOBILE_ATLAS: (&str, usize, f32, f32) = ("cards-mobile-atlas.png", 9, 450.0, 630.0);

pub struct Assets {
    /// Desktop card-face atlas; `None` if absent (renderer falls back to procedural).
    cards: Option<Atlas>,
    /// Higher-legibility mobile atlas, preferred on mobile/touch; may be `None`.
    cards_mobile: Option<Atlas>,
    pub back: Option<Texture2D>,
    pub logo: Option<Texture2D>,
    /// A bundled legible font for all GUI text; `None` falls back to the built-in.
    pub font: Option<Font>,
}

impl Assets {
    /// The face atlas texture and this card's source rect, preferring the mobile
    /// atlas when `mobile` is set and present, else the desktop atlas. `None` means
    /// the renderer should fall back to a procedurally drawn card.
    pub fn face(&self, rank: Rank, suit: Suit, mobile: bool) -> Option<(&Texture2D, Rect)> {
        let atlas = if mobile {
            self.cards_mobile.as_ref().or(self.cards.as_ref())
        } else {
            self.cards.as_ref()
        }?;
        Some((&atlas.tex, atlas.source(rank, suit)))
    }
}

/// Index of a suit in the atlas grid (matches `Suit::ALL`: C, D, H, S).
fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    }
}

/// Which asset a fetched byte blob belongs to.
#[derive(Clone, Copy)]
enum Slot {
    Font,
    Logo,
    Back,
    DeskAtlas,
    MobileAtlas,
}

#[derive(Default)]
struct Shared {
    /// Fetched byte blobs awaiting (main-thread) decode.
    queue: Vec<(Slot, Vec<u8>)>,
    /// Files fetched so far (whether or not they existed).
    fetched: usize,
    total: usize,
    fetch_done: bool,
}

/// Drives asset loading: a background coroutine fetches bytes; `poll` decodes a
/// few per frame and yields the finished `Assets` when complete.
pub struct Loader {
    shared: Arc<Mutex<Shared>>,
    _co: Coroutine,
    cards: Option<Atlas>,
    cards_mobile: Option<Atlas>,
    back: Option<Texture2D>,
    logo: Option<Texture2D>,
    font: Option<Font>,
}

impl Loader {
    pub fn start() -> Loader {
        // On native this prefixes `assets/`; on web assets are served next to the
        // page, so the same relative paths resolve.
        set_pc_assets_folder("assets");

        // font, logo, back, then the two atlases — a handful of files, not ~100.
        let jobs: Vec<(Slot, &str)> = vec![
            (Slot::Font, "fonts/ui.ttf"),
            (Slot::Logo, "king-logo.jpg"),
            (Slot::Back, "back.png"),
            (Slot::DeskAtlas, DESKTOP_ATLAS.0),
            (Slot::MobileAtlas, MOBILE_ATLAS.0),
        ];
        let shared = Arc::new(Mutex::new(Shared {
            total: jobs.len(),
            ..Default::default()
        }));

        let s = shared.clone();
        let _co = start_coroutine(async move {
            for (slot, path) in jobs {
                if let Ok(bytes) = load_file(path).await {
                    s.lock().unwrap().queue.push((slot, bytes));
                }
                s.lock().unwrap().fetched += 1;
            }
            s.lock().unwrap().fetch_done = true;
        });

        Loader {
            shared,
            _co,
            cards: None,
            cards_mobile: None,
            back: None,
            logo: None,
            font: None,
        }
    }

    /// (fetched, total) for the progress bar.
    pub fn progress(&self) -> (usize, usize) {
        let g = self.shared.lock().unwrap();
        (g.fetched, g.total)
    }

    /// Decode up to `budget` fetched blobs into textures/font. Returns the
    /// finished `Assets` once every blob has been fetched and decoded.
    pub fn poll(&mut self, budget: usize) -> Option<Assets> {
        let batch: Vec<(Slot, Vec<u8>)> = {
            let mut g = self.shared.lock().unwrap();
            let n = budget.min(g.queue.len());
            let start = g.queue.len() - n;
            g.queue.split_off(start)
        };
        for (slot, bytes) in batch {
            match slot {
                Slot::Font => self.font = load_ttf_font_from_bytes(&bytes).ok(),
                Slot::Logo => self.logo = Some(decode(&bytes)),
                Slot::Back => self.back = Some(decode(&bytes)),
                Slot::DeskAtlas => {
                    let (_, cols, cw, ch) = DESKTOP_ATLAS;
                    self.cards = Some(Atlas::new(decode(&bytes), cols, cw, ch));
                }
                Slot::MobileAtlas => {
                    let (_, cols, cw, ch) = MOBILE_ATLAS;
                    self.cards_mobile = Some(Atlas::new(decode(&bytes), cols, cw, ch));
                }
            }
        }

        let (fetch_done, empty) = {
            let g = self.shared.lock().unwrap();
            (g.fetch_done, g.queue.is_empty())
        };
        if fetch_done && empty {
            Some(Assets {
                cards: self.cards.take(),
                cards_mobile: self.cards_mobile.take(),
                back: self.back.take(),
                logo: self.logo.take(),
                font: self.font.take(),
            })
        } else {
            None
        }
    }
}

/// Decode image bytes (PNG or JPEG) into a linear-filtered texture.
fn decode(bytes: &[u8]) -> Texture2D {
    let tex = Texture2D::from_file_with_format(bytes, None);
    tex.set_filter(FilterMode::Linear);
    tex
}
