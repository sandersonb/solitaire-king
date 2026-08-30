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

impl Assets {
    pub async fn load() -> Assets {
        // On native this prefixes `assets/`; on web assets are served next to the
        // page, so the same relative paths resolve.
        set_pc_assets_folder("assets");

        let mut cards = HashMap::new();
        let mut cards_mobile = HashMap::new();
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                let name = format!("{}{}.png", rank.label(), suit_code(suit));
                if let Ok(tex) = load_texture(&format!("cards/{name}")).await {
                    tex.set_filter(FilterMode::Linear);
                    cards.insert((rank, suit), tex);
                }
                // The mobile set is optional; absence falls back to `cards`.
                if let Ok(tex) = load_texture(&format!("cards-mobile/{name}")).await {
                    tex.set_filter(FilterMode::Linear);
                    cards_mobile.insert((rank, suit), tex);
                }
            }
        }
        let back = load_texture("cards/back.png").await.ok();
        let logo = load_texture("king-logo.png").await.ok();
        for tex in back.iter().chain(logo.iter()) {
            tex.set_filter(FilterMode::Linear);
        }

        // A bundled legible font; absence is tolerated (built-in font is used).
        let font = load_ttf_font("fonts/ui.ttf").await.ok();

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
