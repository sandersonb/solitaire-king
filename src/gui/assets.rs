//! Asset loading. Raw file bytes are fetched in a background coroutine while the
//! main loop draws the loading screen every frame and decodes a few blobs into
//! textures per frame. This matters on web: decoding in the async pre-loop parked
//! on each `fetch`, so RAF ticks passed with nothing drawn and the canvas showed
//! black between frames (a flicker on real GPUs). Driving the load from the main
//! loop means every frame presents a drawn frame. Missing assets are tolerated —
//! the renderer falls back to procedural cards.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use macroquad::experimental::coroutines::{start_coroutine, Coroutine};
use macroquad::prelude::*;

use klondike::{Rank, Suit};

pub struct Assets {
    pub cards: HashMap<(Rank, Suit), Texture2D>,
    /// A higher-legibility card set preferred on mobile/touch; may be empty.
    pub cards_mobile: HashMap<(Rank, Suit), Texture2D>,
    pub back: Option<Texture2D>,
    pub logo: Option<Texture2D>,
    /// A bundled legible font for all GUI text; `None` falls back to the built-in.
    pub font: Option<Font>,
}

impl Assets {
    /// The face texture for `(rank, suit)`, preferring the mobile set when
    /// `mobile` is set and it is present, else the desktop set. `None` means the
    /// renderer should fall back to a procedurally drawn card.
    pub fn face(&self, rank: Rank, suit: Suit, mobile: bool) -> Option<&Texture2D> {
        if mobile {
            if let Some(tex) = self.cards_mobile.get(&(rank, suit)) {
                return Some(tex);
            }
        }
        self.cards.get(&(rank, suit))
    }
}

/// Filename code for a suit (matches common PD deck naming: AS, 10H, ...).
fn suit_code(suit: Suit) -> char {
    match suit {
        Suit::Clubs => 'C',
        Suit::Diamonds => 'D',
        Suit::Hearts => 'H',
        Suit::Spades => 'S',
    }
}

/// Which asset a fetched byte blob belongs to.
#[derive(Clone, Copy)]
enum Slot {
    Font,
    Logo,
    Back,
    Card(Rank, Suit),
    Mobile(Rank, Suit),
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
    cards: HashMap<(Rank, Suit), Texture2D>,
    cards_mobile: HashMap<(Rank, Suit), Texture2D>,
    back: Option<Texture2D>,
    logo: Option<Texture2D>,
    font: Option<Font>,
}

impl Loader {
    pub fn start() -> Loader {
        // On native this prefixes `assets/`; on web assets are served next to the
        // page, so the same relative paths resolve.
        set_pc_assets_folder("assets");

        let total = 3 + Suit::ALL.len() * Rank::ALL.len() * 2;
        let shared = Arc::new(Mutex::new(Shared {
            total,
            ..Default::default()
        }));

        let s = shared.clone();
        let _co = start_coroutine(async move {
            // font, logo, back first (so the splash has them early), then cards.
            let mut jobs: Vec<(Slot, String)> = vec![
                (Slot::Font, "fonts/ui.ttf".to_string()),
                (Slot::Logo, "king-logo.png".to_string()),
                (Slot::Back, "cards/back.png".to_string()),
            ];
            for suit in Suit::ALL {
                for rank in Rank::ALL {
                    let name = format!("{}{}.png", rank.label(), suit_code(suit));
                    jobs.push((Slot::Card(rank, suit), format!("cards/{name}")));
                    jobs.push((Slot::Mobile(rank, suit), format!("cards-mobile/{name}")));
                }
            }
            for (slot, path) in jobs {
                if let Ok(bytes) = load_file(&path).await {
                    s.lock().unwrap().queue.push((slot, bytes));
                }
                s.lock().unwrap().fetched += 1;
            }
            s.lock().unwrap().fetch_done = true;
        });

        Loader {
            shared,
            _co,
            cards: HashMap::new(),
            cards_mobile: HashMap::new(),
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
                Slot::Card(r, s) => {
                    self.cards.insert((r, s), decode(&bytes));
                }
                Slot::Mobile(r, s) => {
                    self.cards_mobile.insert((r, s), decode(&bytes));
                }
            }
        }

        let (fetch_done, empty) = {
            let g = self.shared.lock().unwrap();
            (g.fetch_done, g.queue.is_empty())
        };
        if fetch_done && empty {
            Some(Assets {
                cards: std::mem::take(&mut self.cards),
                cards_mobile: std::mem::take(&mut self.cards_mobile),
                back: self.back.take(),
                logo: self.logo.take(),
                font: self.font.take(),
            })
        } else {
            None
        }
    }
}

/// Decode PNG bytes into a linear-filtered texture.
fn decode(bytes: &[u8]) -> Texture2D {
    let tex = Texture2D::from_file_with_format(bytes, None);
    tex.set_filter(FilterMode::Linear);
    tex
}
