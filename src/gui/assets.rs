//! Async asset loading (card sprites, back, logo). Missing assets are tolerated
//! — the renderer falls back to procedural cards, so the game is always playable.

use std::collections::HashMap;

use macroquad::prelude::*;

use klondike::{Rank, Suit};

pub struct Assets {
    pub cards: HashMap<(Rank, Suit), Texture2D>,
    pub back: Option<Texture2D>,
    pub logo: Option<Texture2D>,
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
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                let path = format!("cards/{}{}.png", rank.label(), suit_code(suit));
                if let Ok(tex) = load_texture(&path).await {
                    tex.set_filter(FilterMode::Linear);
                    cards.insert((rank, suit), tex);
                }
            }
        }
        let back = load_texture("cards/back.png").await.ok();
        let logo = load_texture("king-logo.png").await.ok();
        for tex in back.iter().chain(logo.iter()) {
            tex.set_filter(FilterMode::Linear);
        }

        Assets { cards, back, logo }
    }
}
