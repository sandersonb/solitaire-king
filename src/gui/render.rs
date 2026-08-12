//! All drawing: the splash screen and the game board. Cards render from sprites
//! when present, else procedurally, so the game is always visible.

use macroquad::prelude::*;

use crate::assets::Assets;
use crate::input::Source;
use crate::layout::{Layout, NUM_TABLEAU};
use crate::session::Session;
use klondike::{Card, Color as CardColor, Suit};

const CREAM: Color = Color::new(0.97, 0.96, 0.90, 1.0);
const TABLE: Color = Color::new(0.10, 0.45, 0.22, 1.0);
const BACK_BLUE: Color = Color::new(0.15, 0.25, 0.55, 1.0);
const HILITE: Color = Color::new(1.0, 0.9, 0.2, 1.0);

fn suit_letter(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Hearts => "H",
        Suit::Spades => "S",
    }
}

fn suit_color(card: Card) -> Color {
    match card.color() {
        CardColor::Red => Color::new(0.75, 0.10, 0.10, 1.0),
        CardColor::Black => Color::new(0.05, 0.05, 0.05, 1.0),
    }
}

fn tex_params(w: f32, h: f32) -> DrawTextureParams {
    DrawTextureParams {
        dest_size: Some(vec2(w, h)),
        ..Default::default()
    }
}

const BORDER: Color = Color::new(0.15, 0.15, 0.15, 1.0);

fn corner_radius(r: Rect) -> f32 {
    r.w * 0.09
}

/// Fill a rounded rectangle (two rects + four corner circles).
fn round_rect(r: Rect, radius: f32, color: Color) {
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0);
    draw_rectangle(r.x + rad, r.y, r.w - 2.0 * rad, r.h, color);
    draw_rectangle(r.x, r.y + rad, r.w, r.h - 2.0 * rad, color);
    draw_circle(r.x + rad, r.y + rad, rad, color);
    draw_circle(r.x + r.w - rad, r.y + rad, rad, color);
    draw_circle(r.x + rad, r.y + r.h - rad, rad, color);
    draw_circle(r.x + r.w - rad, r.y + r.h - rad, rad, color);
}

/// A rounded white card face with a dark border — drawn behind every card so
/// transparent sprites read as cards and stacked cards are countable.
fn card_frame(r: Rect) {
    let b = (r.w * 0.02).max(1.5);
    let outer = Rect::new(r.x - b, r.y - b, r.w + 2.0 * b, r.h + 2.0 * b);
    round_rect(outer, corner_radius(r) + b, BORDER);
    round_rect(r, corner_radius(r), WHITE);
}

/// An empty-pile placeholder: a subtle rounded slot.
fn draw_placeholder(r: Rect) {
    round_rect(r, corner_radius(r), Color::new(1.0, 1.0, 1.0, 0.10));
}

/// Draw a face-down card back into `r` (framed, so backs in a stack are countable).
fn draw_back(assets: &Assets, r: Rect) {
    card_frame(r);
    if let Some(back) = &assets.back {
        draw_texture_ex(back, r.x, r.y, WHITE, tex_params(r.w, r.h));
    } else {
        round_rect(r, corner_radius(r), BACK_BLUE);
    }
}

/// Draw one card (face-up or face-down) into `r`.
fn draw_card(assets: &Assets, r: Rect, card: Card) {
    if !card.face_up {
        draw_back(assets, r);
        return;
    }
    card_frame(r);
    if let Some(tex) = assets.cards.get(&(card.rank, card.suit)) {
        draw_texture_ex(tex, r.x, r.y, WHITE, tex_params(r.w, r.h));
        return;
    }
    // Procedural fallback: rank + suit letter, colored by suit, on the white frame.
    let color = suit_color(card);
    let fs = (r.h * 0.28).round();
    let label = format!("{}{}", card.rank.label(), suit_letter(card.suit));
    draw_text(&label, r.x + r.w * 0.08, r.y + fs, fs, color);
    let big = (r.h * 0.5).round();
    draw_text(
        suit_letter(card.suit),
        r.x + r.w * 0.30,
        r.y + r.h * 0.72,
        big,
        color,
    );
}

fn draw_highlight(r: Rect) {
    draw_rectangle_lines(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 4.0, HILITE);
}

/// Draw the splash screen: logo, title, version, build date, author.
pub fn splash(assets: &Assets) {
    clear_background(TABLE);
    let sw = screen_width();
    let sh = screen_height();

    let center = |text: &str, y: f32, fs: f32, color: Color| {
        let dims = measure_text(text, None, fs as u16, 1.0);
        draw_text(text, sw / 2.0 - dims.width / 2.0, y, fs, color);
    };

    // A modest logo up top; text flows below it (never overlaps).
    let mut y = sh * 0.30;
    if let Some(logo) = &assets.logo {
        let lh = (sh * 0.30).min(240.0);
        let lw = lh * (logo.width() / logo.height());
        let ly = sh * 0.08;
        draw_texture_ex(logo, sw / 2.0 - lw / 2.0, ly, WHITE, tex_params(lw, lh));
        y = ly + lh + sh * 0.06;
    }

    center("Klondike Solitaire", y, 44.0, CREAM);
    center(
        &format!("v{}", env!("CARGO_PKG_VERSION")),
        y + 40.0,
        24.0,
        CREAM,
    );
    center(
        &format!("build {}", env!("BUILD_DATE")),
        y + 72.0,
        22.0,
        CREAM,
    );
    center(
        &format!("by {}", env!("CARGO_PKG_AUTHORS")),
        y + 102.0,
        22.0,
        CREAM,
    );
    center(
        "click or press any key to play",
        sh * 0.92,
        20.0,
        Color::new(1.0, 1.0, 1.0, 0.7),
    );
}

fn fmt_time(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

/// Draw the full board for the current session.
pub fn board(session: &Session, assets: &Assets, layout: &Layout, selection: Option<Source>) {
    clear_background(TABLE);
    let state = &session.state;

    // Stock: a face-down back when it has cards, else a recycle placeholder.
    if state.stock.is_empty() {
        draw_placeholder(layout.stock);
        draw_text(
            "R",
            layout.stock.x + layout.stock.w * 0.35,
            layout.stock.y + layout.stock.h * 0.62,
            layout.stock.h * 0.5,
            CREAM,
        );
    } else {
        draw_back(assets, layout.stock);
    }

    // Waste: up to the last three, fanned right so the top (movable) card is last.
    let waste = state.waste.cards();
    let waste_rect = |i: usize| {
        Rect::new(
            layout.waste.x + i as f32 * layout.card_w * 0.28,
            layout.waste.y,
            layout.waste.w,
            layout.waste.h,
        )
    };
    if waste.is_empty() {
        draw_placeholder(layout.waste);
    } else {
        let shown = waste.len().min(3);
        for (i, card) in waste[waste.len() - shown..].iter().enumerate() {
            draw_card(assets, waste_rect(i), *card);
        }
        // Highlight the top card (the one that actually moves), not the base.
        if selection == Some(Source::Waste) {
            draw_highlight(waste_rect(shown - 1));
        }
    }

    // Foundations.
    for (i, r) in layout.foundations.iter().enumerate() {
        match state.foundations[i].top() {
            Some(card) => draw_card(assets, *r, card),
            None => draw_placeholder(*r),
        }
        if selection == Some(Source::Foundation(i)) {
            draw_highlight(*r);
        }
    }

    // Tableau columns.
    for col in 0..NUM_TABLEAU {
        let cards = state.tableau[col].cards();
        if cards.is_empty() {
            draw_placeholder(layout.tableau[col]);
            continue;
        }
        for (index, card) in cards.iter().enumerate() {
            draw_card(assets, layout.tableau_card_rect(col, index), *card);
        }
        if let Some(Source::TableauRun { col: sc, index }) = selection {
            if sc == col {
                for i in index..cards.len() {
                    draw_highlight(layout.tableau_card_rect(col, i));
                }
            }
        }
    }

    draw_status(session);

    if session.is_won() {
        draw_win_banner(session, assets);
    }
}

fn draw_status(session: &Session) {
    let line = format!(
        "seed {}    moves {}    score {}    time {}",
        session.seed(),
        session.move_count(),
        session.score(),
        fmt_time(session.elapsed_secs()),
    );
    draw_text(&line, 12.0, 24.0, 24.0, CREAM);
    if let Some(msg) = session.message() {
        draw_text(msg, 12.0, screen_height() - 30.0, 22.0, HILITE);
    }
    draw_text(
        "click select+move · dbl-click/Enter auto · U undo · R redo · N new",
        12.0,
        screen_height() - 8.0,
        18.0,
        Color::new(1.0, 1.0, 1.0, 0.6),
    );
}

fn draw_win_banner(session: &Session, assets: &Assets) {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55));
    if let Some(logo) = &assets.logo {
        let lw = (sw * 0.16).min(180.0);
        let lh = lw * (logo.height() / logo.width());
        draw_texture_ex(
            logo,
            sw / 2.0 - lw / 2.0,
            sh * 0.22,
            WHITE,
            tex_params(lw, lh),
        );
    }
    let center = |text: &str, y: f32, fs: f32, color: Color| {
        let dims = measure_text(text, None, fs as u16, 1.0);
        draw_text(text, sw / 2.0 - dims.width / 2.0, y, fs, color);
    };
    center("You win!", sh * 0.52, 56.0, HILITE);
    center(
        &format!(
            "final score {}   in {}",
            session.final_score(),
            fmt_time(session.elapsed_secs())
        ),
        sh * 0.60,
        28.0,
        CREAM,
    );
    center("press N for a new game", sh * 0.68, 22.0, CREAM);
}
