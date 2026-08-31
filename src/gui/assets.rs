//! Async asset loading (card sprites, back, logo). Missing assets are tolerated
//! — the renderer falls back to procedural cards, so the game is always playable.

use std::collections::HashMap;

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

/// Filename code for a suit (matches common PD deck naming: AS, 10H, ...).
fn suit_code(suit: Suit) -> char {
    match suit {
        Suit::Clubs => 'C',
        Suit::Diamonds => 'D',
        Suit::Hearts => 'H',
        Suit::Spades => 'S',
    }
}

/// Background for the loading screen (matches the board felt).
const LOADING_BG: Color = Color::new(0.10, 0.45, 0.22, 1.0);

/// Draw a loading progress screen and yield a frame. The first `next_frame`
/// here paints the (opaque) canvas, covering the page's HTML spinner.
async fn loading_frame(done: usize, total: usize, font: Option<&Font>) {
    clear_background(LOADING_BG);
    let sw = screen_width();
    let sh = screen_height();
    let frac = (done as f32 / total as f32).clamp(0.0, 1.0);

    let title = "Loading…";
    let fs = 32.0;
    let dims = measure_text(title, font, fs as u16, 1.0);
    draw_text_ex(
        title,
        sw / 2.0 - dims.width / 2.0,
        sh * 0.44,
        TextParams {
            font,
            font_size: fs as u16,
            color: Color::new(0.97, 0.96, 0.90, 1.0),
            ..Default::default()
        },
    );

    let bw = (sw * 0.5).clamp(160.0, 420.0);
    let bh = 16.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh * 0.5;
    draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(0.85, 0.90, 0.82, 1.0));
    draw_rectangle(bx + 2.0, by + 2.0, (bw - 4.0) * frac, bh - 4.0, WHITE);

    next_frame().await;
}

impl Assets {
    pub async fn load() -> Assets {
        // On native this prefixes `assets/`; on web assets are served next to the
        // page, so the same relative paths resolve.
        set_pc_assets_folder("assets");

        // font (1) + logo (1) + back (1) + cards + mobile cards.
        let total = 3 + Suit::ALL.len() * Rank::ALL.len() * 2;
        let mut done = 0usize;

        // Paint the loading screen immediately (built-in font), before the first
        // asset fetch, so the now-opaque WebGL canvas doesn't flash black between
        // context creation and the first drawn frame on web.
        loading_frame(0, total, None).await;

        // Load the font first so the loading screen (and splash) can use it;
        // absence is tolerated (built-in font is used).
        let font = load_ttf_font("fonts/ui.ttf").await.ok();
        done += 1;
        loading_frame(done, total, font.as_ref()).await;

        let logo = load_texture("king-logo.png").await.ok();
        done += 1;
        let back = load_texture("cards/back.png").await.ok();
        done += 1;
        for tex in back.iter().chain(logo.iter()) {
            tex.set_filter(FilterMode::Linear);
        }
        loading_frame(done, total, font.as_ref()).await;

        let mut cards = HashMap::new();
        let mut cards_mobile = HashMap::new();
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                let name = format!("{}{}.png", rank.label(), suit_code(suit));
                if let Ok(tex) = load_texture(&format!("cards/{name}")).await {
                    tex.set_filter(FilterMode::Linear);
                    cards.insert((rank, suit), tex);
                }
                done += 1;
                // The mobile set is optional; absence falls back to `cards`.
                if let Ok(tex) = load_texture(&format!("cards-mobile/{name}")).await {
                    tex.set_filter(FilterMode::Linear);
                    cards_mobile.insert((rank, suit), tex);
                }
                done += 1;
                // Draw the bar and yield after every card so the OS run loop is
                // serviced between the (heavy) PNG decodes — otherwise macOS shows
                // a beachball and the bar jumps instead of ticking.
                loading_frame(done, total, font.as_ref()).await;
            }
        }
        loading_frame(total, total, font.as_ref()).await;

        Assets {
            cards,
            cards_mobile,
            back,
            logo,
            font,
        }
    }

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
