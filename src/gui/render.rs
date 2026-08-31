//! All drawing: the splash screen and the game board. Cards render from sprites
//! when present, else procedurally, so the game is always visible.

use macroquad::prelude::*;

use crate::anim::Animator;
use crate::assets::Assets;
use crate::input::{Drag, Pile, Source};
use crate::layout::{ButtonId, Layout, NUM_TABLEAU};
use crate::session::Session;
use crate::solver::Status;
use klondike::{Card, Color as CardColor, Suit};

const CREAM: Color = Color::new(0.97, 0.96, 0.90, 1.0);
const TABLE: Color = Color::new(0.10, 0.45, 0.22, 1.0);
const BACK_BLUE: Color = Color::new(0.15, 0.25, 0.55, 1.0);
const HILITE: Color = Color::new(1.0, 0.9, 0.2, 1.0);

/// Draw text with the bundled font when present, else the built-in font.
fn text(font: Option<&Font>, s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font,
            font_size: size.round().max(1.0) as u16,
            color,
            ..Default::default()
        },
    );
}

/// Measure text with the same font the renderer draws with, for centering.
fn measure(font: Option<&Font>, s: &str, size: f32) -> TextDimensions {
    measure_text(s, font, size.round().max(1.0) as u16, 1.0)
}

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

/// Draw one card (face-up or face-down) into `r`. `mobile` prefers the
/// higher-legibility mobile card set when it is present.
fn draw_card(assets: &Assets, r: Rect, card: Card, mobile: bool) {
    if !card.face_up {
        draw_back(assets, r);
        return;
    }
    card_frame(r);
    if let Some(tex) = assets.face(card.rank, card.suit, mobile) {
        draw_texture_ex(tex, r.x, r.y, WHITE, tex_params(r.w, r.h));
        return;
    }
    // Procedural fallback: rank + suit letter, colored by suit, on the white frame.
    let color = suit_color(card);
    let fs = (r.h * 0.28).round();
    let label = format!("{}{}", card.rank.label(), suit_letter(card.suit));
    text(
        assets.font.as_ref(),
        &label,
        r.x + r.w * 0.08,
        r.y + fs,
        fs,
        color,
    );
    let big = (r.h * 0.5).round();
    text(
        assets.font.as_ref(),
        suit_letter(card.suit),
        r.x + r.w * 0.30,
        r.y + r.h * 0.72,
        big,
        color,
    );
}

/// Draw a downward-fanned run of cards starting at top-left `at`.
fn draw_run(assets: &Assets, at: Vec2, cards: &[Card], card_w: f32, fan_dy: f32, mobile: bool) {
    let card_h = card_w * 1.4;
    for (i, card) in cards.iter().enumerate() {
        let r = Rect::new(at.x, at.y + i as f32 * fan_dy, card_w, card_h);
        draw_card(assets, r, *card, mobile);
    }
}

/// Draw the splash screen: logo, title, version, build date, author.
pub fn splash(assets: &Assets) {
    clear_background(TABLE);
    let sw = screen_width();
    let sh = screen_height();

    let font = assets.font.as_ref();
    let center = |s: &str, y: f32, fs: f32, color: Color| {
        let dims = measure(font, s, fs);
        text(font, s, sw / 2.0 - dims.width / 2.0, y, fs, color);
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

/// Draw the full board for the current session. `drag` is the card/run being
/// carried by the pointer (its source cards are hidden and drawn on top);
/// `anim` holds in-flight snap animations (whose destination cards are hidden
/// in the static board until they land).
pub fn board(
    session: &Session,
    assets: &Assets,
    layout: &Layout,
    drag: Option<&Drag>,
    anim: &Animator,
    show_seed: bool,
) {
    clear_background(TABLE);
    let state = &session.state;
    let mobile = layout.mobile;

    // How many top cards of a pile to hide: those flying in (animations) plus,
    // for the drag source, the run being carried.
    let drag_src = drag.map(|d| d.source);
    let hidden = |pile: Pile, drag_hides: usize| anim.suppressed(pile) + drag_hides;

    // Stock: a face-down back when it has cards, else a recycle placeholder.
    if state.stock.is_empty() {
        draw_placeholder(layout.stock);
        text(
            assets.font.as_ref(),
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
    // Keep the fan anchored to the full pile while a card is lifted off the top
    // (dragged or animating): hide the top card(s) in place rather than shifting
    // the window, so the cards beneath don't shuffle and reveal a new one.
    let waste_drag = usize::from(drag_src == Some(Source::Waste));
    let hide_top = waste_drag + anim.suppressed(Pile::Waste);
    if waste.is_empty() {
        draw_placeholder(layout.waste);
    } else {
        let shown = waste.len().min(3);
        let mut drawn = 0;
        for (i, card) in waste[waste.len() - shown..].iter().enumerate() {
            // Top card is at i == shown - 1; skip the top `hide_top` of them.
            if i + hide_top >= shown {
                continue;
            }
            draw_card(assets, waste_rect(i), *card, mobile);
            drawn += 1;
        }
        if drawn == 0 {
            draw_placeholder(layout.waste);
        }
    }

    // Foundations.
    for (i, r) in layout.foundations.iter().enumerate() {
        let cards = state.foundations[i].cards();
        let drag_here = usize::from(drag_src == Some(Source::Foundation(i)));
        let visible = cards
            .len()
            .saturating_sub(hidden(Pile::Foundation(i), drag_here));
        match visible.checked_sub(1).and_then(|idx| cards.get(idx)) {
            Some(card) => draw_card(assets, *r, *card, mobile),
            None => draw_placeholder(*r),
        }
    }

    // Tableau columns.
    for col in 0..NUM_TABLEAU {
        let cards = state.tableau[col].cards();
        let drag_here = match drag_src {
            Some(Source::TableauRun { col: sc, index }) if sc == col => cards.len() - index,
            _ => 0,
        };
        let visible = cards
            .len()
            .saturating_sub(hidden(Pile::Tableau(col), drag_here));
        if visible == 0 {
            draw_placeholder(layout.tableau[col]);
            continue;
        }
        for (index, card) in cards[..visible].iter().enumerate() {
            draw_card(assets, layout.tableau_card_rect(col, index), *card, mobile);
        }
    }

    // In-flight snap animations, on top of the board.
    let now = get_time();
    for a in &anim.anims {
        draw_run(assets, a.pos(now), &a.cards, a.card_w, a.fan_dy, mobile);
    }

    // The dragged run follows the pointer, lifted + enlarged on touch so a
    // finger doesn't occlude it.
    if let Some(d) = drag {
        let scale = if mobile { 1.15 } else { 1.0 };
        let cw = layout.card_w * scale;
        let lift = if mobile { layout.card_w * 0.9 } else { 0.0 };
        let tl = d.top_left();
        draw_run(
            assets,
            vec2(tl.x, tl.y - lift),
            &d.cards,
            cw,
            layout.fan_dy * scale,
            mobile,
        );
    }

    draw_status(session, assets, show_seed);

    // The win banner dims the board; the control bar is drawn on top of it so its
    // buttons (New, Settings) stay visible and usable after a win.
    if session.is_won() {
        draw_win_banner(session, assets);
    }
    draw_control_bar(assets, layout, session.is_won());
}

const BTN_BG: Color = Color::new(0.08, 0.30, 0.16, 1.0);
const BTN_EDGE: Color = Color::new(0.85, 0.90, 0.82, 1.0);

/// Draw the on-screen control bar, highlighting the pressed control. `undo_off`
/// dims the Undo/Redo button (undo/redo are disabled after a win).
fn draw_control_bar(assets: &Assets, layout: &Layout, undo_off: bool) {
    let font = assets.font.as_ref();
    // A subtle strip separating the controls from the board.
    draw_rectangle(
        layout.bar.x,
        layout.bar.y,
        layout.bar.w,
        layout.bar.h,
        Color::new(0.06, 0.24, 0.13, 1.0),
    );
    let (px, py) = mouse_position();
    for (id, r) in &layout.buttons {
        let disabled = *id == ButtonId::UndoRedo && undo_off;
        let pressed = !disabled
            && is_mouse_button_down(MouseButton::Left)
            && px >= r.x
            && px <= r.x + r.w
            && py >= r.y
            && py <= r.y + r.h;
        let bg = if pressed {
            Color::new(0.14, 0.45, 0.24, 1.0)
        } else if disabled {
            Color::new(0.06, 0.18, 0.10, 1.0)
        } else {
            BTN_BG
        };
        round_rect(*r, r.h * 0.22, bg);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, BTN_EDGE);
        let label = match id {
            ButtonId::UndoRedo => "Undo",
            ButtonId::New => "New",
            ButtonId::Settings => "Settings",
        };
        let fg = if disabled {
            Color::new(1.0, 1.0, 1.0, 0.35)
        } else {
            CREAM
        };
        let fs = (r.h * 0.44).round();
        let dims = measure(font, label, fs);
        text(
            font,
            label,
            r.x + (r.w - dims.width) / 2.0,
            r.y + (r.h + dims.height) / 2.0,
            fs,
            fg,
        );
    }
}

/// A choice offered by the unwinnable dialog.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DialogChoice {
    Continue,
    NewGame,
}

/// Draw the solvability indicator badge at the left of the control bar, with a
/// distinct visual per status. Drawn as vector shapes so it needs no special
/// font glyph coverage.
pub fn solver_indicator(assets: &Assets, layout: &Layout, status: Status, disabled: bool) {
    let r = layout.indicator;
    let s = r.w;
    let cx = r.x + r.w / 2.0;
    let cy = r.y + r.h / 2.0;
    let lw = (s * 0.09).max(2.0);

    // Disabled: a muted badge with a dash, and no status visual.
    if disabled {
        round_rect(r, s * 0.22, Color::new(0.30, 0.33, 0.30, 1.0));
        draw_line(
            cx - s * 0.22,
            cy,
            cx + s * 0.22,
            cy,
            lw,
            Color::new(0.8, 0.8, 0.8, 0.7),
        );
        return;
    }

    let green = Color::new(0.20, 0.65, 0.30, 1.0);
    let red = Color::new(0.80, 0.20, 0.20, 1.0);
    let blue = Color::new(0.25, 0.45, 0.75, 1.0);
    let gray = Color::new(0.45, 0.48, 0.45, 1.0);
    let bg = match status {
        Status::Solvable => green,
        Status::Unwinnable => red,
        Status::Checking => blue,
        Status::Unknown | Status::Inconclusive => gray,
    };
    round_rect(r, s * 0.22, bg);

    match status {
        Status::Solvable => {
            // Check mark.
            draw_line(
                cx - s * 0.22,
                cy + s * 0.02,
                cx - s * 0.05,
                cy + s * 0.20,
                lw,
                WHITE,
            );
            draw_line(
                cx - s * 0.05,
                cy + s * 0.20,
                cx + s * 0.25,
                cy - s * 0.22,
                lw,
                WHITE,
            );
        }
        Status::Unwinnable => {
            // Cross.
            let d = s * 0.22;
            draw_line(cx - d, cy - d, cx + d, cy + d, lw, WHITE);
            draw_line(cx - d, cy + d, cx + d, cy - d, lw, WHITE);
        }
        Status::Checking => {
            // Spinner: a rotating arc of ticks.
            let t = get_time();
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0 + t as f32 * 4.0;
                let alpha = 0.25 + 0.75 * (i as f32 / 7.0);
                let (sx, sy) = (cx + a.cos() * s * 0.16, cy + a.sin() * s * 0.16);
                let (ex, ey) = (cx + a.cos() * s * 0.30, cy + a.sin() * s * 0.30);
                draw_line(sx, sy, ex, ey, lw, Color::new(1.0, 1.0, 1.0, alpha));
            }
        }
        Status::Unknown | Status::Inconclusive => {
            let fs = s * 0.7;
            let dims = measure(assets.font.as_ref(), "?", fs);
            text(
                assets.font.as_ref(),
                "?",
                cx - dims.width / 2.0,
                cy + dims.height / 2.0,
                fs,
                WHITE,
            );
        }
    }
}

/// The Continue / New game button rects for the unwinnable dialog.
pub fn dialog_button_rects() -> [(DialogChoice, Rect); 2] {
    let (sw, sh) = (screen_width(), screen_height());
    let pw = (sw * 0.6).clamp(280.0, 520.0);
    let ph = (sh * 0.32).clamp(160.0, 260.0);
    let px = (sw - pw) / 2.0;
    let py = (sh - ph) / 2.0;
    let bw = pw * 0.40;
    let bh = ph * 0.24;
    let by = py + ph - bh - ph * 0.12;
    let gap = pw * 0.06;
    let total = bw * 2.0 + gap;
    let bx = px + (pw - total) / 2.0;
    [
        (DialogChoice::Continue, Rect::new(bx, by, bw, bh)),
        (DialogChoice::NewGame, Rect::new(bx + bw + gap, by, bw, bh)),
    ]
}

/// Draw the modal "this deal can't be won" dialog with its two buttons.
pub fn unwinnable_dialog(assets: &Assets) {
    let font = assets.font.as_ref();
    let (sw, sh) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55));

    let pw = (sw * 0.6).clamp(280.0, 520.0);
    let ph = (sh * 0.32).clamp(160.0, 260.0);
    let panel = Rect::new((sw - pw) / 2.0, (sh - ph) / 2.0, pw, ph);
    round_rect(panel, 16.0, Color::new(0.12, 0.16, 0.13, 1.0));
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, BTN_EDGE);

    let center = |s: &str, y: f32, fs: f32, color: Color| {
        let dims = measure(font, s, fs);
        text(font, s, sw / 2.0 - dims.width / 2.0, y, fs, color);
    };
    center(
        "No moves can win this deal.",
        panel.y + ph * 0.30,
        28.0,
        CREAM,
    );
    center(
        "Keep playing, or deal a new game?",
        panel.y + ph * 0.46,
        20.0,
        CREAM,
    );

    let (px, py) = mouse_position();
    for (choice, r) in dialog_button_rects() {
        let hot = px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h;
        let bg = if hot && is_mouse_button_down(MouseButton::Left) {
            Color::new(0.14, 0.45, 0.24, 1.0)
        } else {
            BTN_BG
        };
        round_rect(r, r.h * 0.22, bg);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, BTN_EDGE);
        let label = match choice {
            DialogChoice::Continue => "Continue",
            DialogChoice::NewGame => "New game",
        };
        let fs = (r.h * 0.42).round();
        let dims = measure(font, label, fs);
        text(
            font,
            label,
            r.x + (r.w - dims.width) / 2.0,
            r.y + (r.h + dims.height) / 2.0,
            fs,
            CREAM,
        );
    }
}

// --- Solver overlay & Settings dialog ---------------------------------------

/// A button in the solver status overlay.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SolverAction {
    AutoSolve,
    NewGame,
    Close,
}

/// A row in the Settings dialog.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingRow {
    DrawMode,
    Solver,
    Seed,
    Close,
}

/// The centered modal panel rect.
fn panel_rect() -> Rect {
    let (sw, sh) = (screen_width(), screen_height());
    let pw = (sw * 0.62).clamp(300.0, 560.0);
    let ph = (sh * 0.40).clamp(200.0, 320.0);
    Rect::new((sw - pw) / 2.0, (sh - ph) / 2.0, pw, ph)
}

/// Evenly place `n` buttons in a row near the bottom of `panel`.
fn button_row(panel: Rect, n: usize) -> Vec<Rect> {
    if n == 0 {
        return Vec::new();
    }
    let bw = (panel.w * 0.36).min(200.0);
    let bh = (panel.h * 0.20).clamp(40.0, 64.0);
    let gap = panel.w * 0.05;
    let total = bw * n as f32 + gap * (n as f32 - 1.0);
    let bx = panel.x + (panel.w - total) / 2.0;
    let by = panel.y + panel.h - bh - panel.h * 0.10;
    (0..n)
        .map(|i| Rect::new(bx + i as f32 * (bw + gap), by, bw, bh))
        .collect()
}

fn draw_button(assets: &Assets, r: Rect, label: &str) {
    let (px, py) = mouse_position();
    let hot = px >= r.x && px <= r.x + r.w && py >= r.y && py <= r.y + r.h;
    let bg = if hot && is_mouse_button_down(MouseButton::Left) {
        Color::new(0.14, 0.45, 0.24, 1.0)
    } else {
        BTN_BG
    };
    round_rect(r, r.h * 0.22, bg);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, BTN_EDGE);
    let font = assets.font.as_ref();
    let fs = (r.h * 0.42).round();
    let dims = measure(font, label, fs);
    text(
        font,
        label,
        r.x + (r.w - dims.width) / 2.0,
        r.y + (r.h + dims.height) / 2.0,
        fs,
        CREAM,
    );
}

/// The actions (and their rects) offered by the solver overlay for a status.
pub fn solver_overlay_actions(
    status: Status,
    has_solution: bool,
    enabled: bool,
) -> Vec<(SolverAction, Rect)> {
    let acts: Vec<SolverAction> = if !enabled {
        vec![SolverAction::Close]
    } else {
        match status {
            Status::Solvable if has_solution => vec![SolverAction::AutoSolve, SolverAction::Close],
            Status::Unwinnable => vec![SolverAction::NewGame, SolverAction::Close],
            _ => vec![SolverAction::Close],
        }
    };
    let rects = button_row(panel_rect(), acts.len());
    acts.into_iter().zip(rects).collect()
}

/// Draw the solver status overlay for the current status.
pub fn solver_overlay(assets: &Assets, status: Status, sol_len: Option<usize>, enabled: bool) {
    let (sw, sh) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.5));
    let panel = panel_rect();
    round_rect(panel, 16.0, Color::new(0.12, 0.16, 0.13, 1.0));
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, BTN_EDGE);

    let font = assets.font.as_ref();
    let center = |s: &str, y: f32, fs: f32, color: Color| {
        let dims = measure(font, s, fs);
        text(font, s, sw / 2.0 - dims.width / 2.0, y, fs, color);
    };

    let (title, sub) = if !enabled {
        (
            "Solver is off".to_string(),
            "Enable the background solver in Settings.".to_string(),
        )
    } else {
        match status {
            Status::Solvable => (
                "A solution exists".to_string(),
                match sol_len {
                    Some(n) => format!("Win in {n} moves. Auto-solve to watch it play out."),
                    None => "Auto-solve will be available once the line is found.".to_string(),
                },
            ),
            Status::Unwinnable => (
                "No solution".to_string(),
                "This deal can't be won — but undoing may reopen a solution.".to_string(),
            ),
            Status::Checking => (
                "Checking…".to_string(),
                "The solver is evaluating this position.".to_string(),
            ),
            Status::Unknown | Status::Inconclusive => (
                "Uncertain".to_string(),
                "Play a few more moves to help determine solvability.".to_string(),
            ),
        }
    };
    center(&title, panel.y + panel.h * 0.28, 30.0, HILITE);
    center(&sub, panel.y + panel.h * 0.46, 19.0, CREAM);

    for (action, r) in solver_overlay_actions(status, sol_len.is_some(), enabled) {
        let label = match action {
            SolverAction::AutoSolve => "Auto-solve",
            SolverAction::NewGame => "New game",
            SolverAction::Close => "Close",
        };
        draw_button(assets, r, label);
    }
}

/// The Settings dialog rows and their rects.
pub fn settings_rows() -> Vec<(SettingRow, Rect)> {
    let panel = panel_rect();
    let m = panel.w * 0.08;
    let rw = panel.w - 2.0 * m;
    let rh = (panel.h * 0.16).clamp(34.0, 52.0);
    let top = panel.y + panel.h * 0.22;
    let step = rh + panel.h * 0.04;
    let mut rows: Vec<(SettingRow, Rect)> =
        [SettingRow::DrawMode, SettingRow::Solver, SettingRow::Seed]
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, Rect::new(panel.x + m, top + i as f32 * step, rw, rh)))
            .collect();
    // Close button at the bottom center.
    let bw = (panel.w * 0.32).min(180.0);
    let bh = (panel.h * 0.18).clamp(38.0, 56.0);
    rows.push((
        SettingRow::Close,
        Rect::new(
            panel.x + (panel.w - bw) / 2.0,
            panel.y + panel.h - bh - panel.h * 0.08,
            bw,
            bh,
        ),
    ));
    rows
}

/// Draw the Settings dialog reflecting the current toggle values.
pub fn settings_overlay(assets: &Assets, draw_three: bool, solver_on: bool, show_seed: bool) {
    let (sw, sh) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.5));
    let panel = panel_rect();
    round_rect(panel, 16.0, Color::new(0.12, 0.16, 0.13, 1.0));
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, BTN_EDGE);
    let font = assets.font.as_ref();
    let title = "Settings";
    let dims = measure(font, title, 28.0);
    text(
        font,
        title,
        sw / 2.0 - dims.width / 2.0,
        panel.y + panel.h * 0.15,
        28.0,
        HILITE,
    );

    for (id, r) in settings_rows() {
        if id == SettingRow::Close {
            draw_button(assets, r, "Close");
            continue;
        }
        // A labelled row with its current value on the right (tap toggles).
        round_rect(r, r.h * 0.2, Color::new(0.08, 0.30, 0.16, 1.0));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, BTN_EDGE);
        let (label, value) = match id {
            SettingRow::DrawMode => ("Draw (next game)", if draw_three { "three" } else { "one" }),
            SettingRow::Solver => ("Background solver", if solver_on { "on" } else { "off" }),
            SettingRow::Seed => ("Show seed", if show_seed { "on" } else { "off" }),
            SettingRow::Close => unreachable!(),
        };
        let fs = (r.h * 0.5).round();
        text(
            font,
            label,
            r.x + r.h * 0.3,
            r.y + (r.h + fs * 0.7) / 2.0,
            fs,
            CREAM,
        );
        let vd = measure(font, value, fs);
        text(
            font,
            value,
            r.x + r.w - vd.width - r.h * 0.3,
            r.y + (r.h + fs * 0.7) / 2.0,
            fs,
            HILITE,
        );
    }
}

fn draw_status(session: &Session, assets: &Assets, show_seed: bool) {
    let font = assets.font.as_ref();
    // Auto-solved games are not scored/timed: show dashes rather than a value.
    let auto = session.is_auto_solving() || session.was_auto_solved();
    let (score, time) = if auto {
        ("—".to_string(), "—".to_string())
    } else {
        (
            session.score().to_string(),
            fmt_time(session.elapsed_secs()),
        )
    };
    let seed = if show_seed {
        format!("seed {}    ", klondike::seed::encode(session.seed()))
    } else {
        String::new()
    };
    let line = format!(
        "{seed}moves {}    score {}    time {}",
        session.move_count(),
        score,
        time,
    );
    text(font, &line, 12.0, 24.0, 24.0, CREAM);
    if let Some(msg) = session.message() {
        text(font, msg, 12.0, screen_height() - 30.0, 22.0, HILITE);
    }
}

fn draw_win_banner(session: &Session, assets: &Assets) {
    let sw = screen_width();
    let sh = screen_height();
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55));

    let font = assets.font.as_ref();
    let center = |s: &str, y: f32, fs: f32, color: Color| {
        let dims = measure(font, s, fs);
        text(font, s, sw / 2.0 - dims.width / 2.0, y, fs, color);
    };

    // Logo up top; text flows strictly below it so nothing overlaps the artwork.
    let ly = sh * 0.12;
    let mut y = ly + sh * 0.06;
    if let Some(logo) = &assets.logo {
        let lh = (sh * 0.26).min(220.0);
        let lw = lh * (logo.width() / logo.height());
        draw_texture_ex(logo, sw / 2.0 - lw / 2.0, ly, WHITE, tex_params(lw, lh));
        y = ly + lh + sh * 0.06;
    }

    if session.was_auto_solved() {
        center("Auto-solved", y, 52.0, HILITE);
        center("(not a scored win)", y + sh * 0.08, 24.0, CREAM);
    } else {
        center("You win!", y, 52.0, HILITE);
        center(
            &format!(
                "final score {}   in {}",
                session.final_score(),
                fmt_time(session.elapsed_secs())
            ),
            y + sh * 0.08,
            28.0,
            CREAM,
        );
    }
    center("New game to play again", y + sh * 0.16, 22.0, CREAM);
}
