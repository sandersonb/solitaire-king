//! Klondike Solitaire — graphical front-end (macroquad; native + WASM).

mod anim;
mod assets;
mod input;
mod layout;
mod render;
mod session;
mod solver;

use macroquad::prelude::*;

use anim::{Animator, CardAnim, SNAP_SECS};
use assets::Assets;
use input::{
    auto_target, hit_test, move_dest, nearest_pile, read_pointer, resolve, source_of, Drag, Hit,
    Pile, Pointer, Source,
};
use klondike::{Card, DrawMode, GameConfig, Move};
use layout::{ButtonId, Layout};
use render::DialogChoice;
use session::Session;
use solver::Assist;

fn window_conf() -> Conf {
    Conf {
        window_title: "Klondike Solitaire".to_string(),
        window_width: 1024,
        window_height: 768,
        high_dpi: true,
        ..Default::default()
    }
}

/// Parse native launch args (`--seed`, `--draw`, `--timed`, `--redeal`). On web
/// there are no args, so this returns defaults.
fn parse_args() -> (Option<u64>, GameConfig) {
    let args: Vec<String> = std::env::args().collect();
    let (mut seed, mut draw, mut timed, mut redeal) = (None, 3u8, false, None);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                i += 1;
                // Accept a proquint string or a raw u64 (same as the CLI).
                seed = args.get(i).and_then(|s| klondike::seed::decode(s));
            }
            "--draw" => {
                i += 1;
                draw = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(3);
            }
            "--timed" => timed = true,
            "--redeal" => {
                i += 1;
                redeal = args.get(i).and_then(|s| s.parse().ok());
            }
            _ => {}
        }
        i += 1;
    }
    let draw_mode = if draw == 1 {
        DrawMode::One
    } else {
        DrawMode::Three
    };
    (
        seed,
        GameConfig {
            draw_mode,
            redeal_limit: redeal,
            timed,
        },
    )
}

/// A time-seeded `u64` (works native and web).
fn random_seed() -> u64 {
    let mut x = (miniquad::date::now() * 1000.0) as u64 ^ 0x9E37_79B9_7F4A_7C15;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

enum App {
    Splash(f64),
    Playing,
}

#[macroquad::main(window_conf)]
async fn main() {
    let (seed_arg, cfg) = parse_args();
    let assets = Assets::load().await;

    let mut session = Session::new(seed_arg.unwrap_or_else(random_seed), cfg);
    let mut game_start = get_time();
    let mut app = App::Splash(get_time());
    let mut drag: Option<Drag> = None;
    let mut anim = Animator::new();
    let mut assist = Assist::new(&session.state);
    let mut last_seed = session.seed();
    let mut last_key = klondike::encode(&session.state);
    let mut last_tap = 0.0f64;
    let mut last_hit: Option<Hit> = None;
    let mut touch_seen = false;

    loop {
        let ptr = read_pointer();
        if !touches().is_empty() {
            touch_seen = true;
        }
        match app {
            App::Splash(since) => {
                render::splash(&assets);
                let shown = get_time() - since;
                let dismiss = ptr.pressed || get_last_key_pressed().is_some() || shown > 3.0;
                if shown > 0.3 && dismiss {
                    app = App::Playing;
                    game_start = get_time();
                }
            }
            App::Playing => {
                let max_col_len = session
                    .state
                    .tableau
                    .iter()
                    .map(|c| c.len())
                    .max()
                    .unwrap_or(1);
                let layout =
                    Layout::compute(screen_width(), screen_height(), max_col_len, touch_seen);
                if !session.is_won() {
                    session.set_elapsed((get_time() - game_start).max(0.0) as u64);
                }
                anim.tick(get_time());
                if assist.dialog_open() {
                    // The dialog is modal: it swallows board input.
                    handle_dialog(
                        &ptr,
                        &mut session,
                        &mut assist,
                        &mut anim,
                        &mut game_start,
                        cfg,
                    );
                } else {
                    handle_input(
                        &mut session,
                        &mut drag,
                        &mut anim,
                        &layout,
                        cfg,
                        &mut game_start,
                        &mut last_tap,
                        &mut last_hit,
                        &ptr,
                    );
                    // Automated playback: when idle, apply and animate a queued move.
                    if drag.is_none() && !anim.is_animating() {
                        if let Some(mv) = anim.next_queued() {
                            play_queued(&mut session, &mut anim, &layout, mv);
                        }
                    }
                }

                // Detect state changes so the assist re-evaluates: a new seed is a
                // fresh deal (reset knowledge); any other position change is a
                // move/undo/redo.
                let seed_now = session.seed();
                let key_now = klondike::encode(&session.state);
                if seed_now != last_seed {
                    assist.reset(&session.state);
                    last_seed = seed_now;
                } else if key_now != last_key {
                    assist.on_state_change(&session.state);
                }
                last_key = key_now;
                if ptr.pressed || ptr.released {
                    assist.note_activity();
                }
                assist.update(&session.state);

                render::board(&session, &assets, &layout, drag.as_ref(), &anim);
                render::solver_indicator(&assets, &layout, assist.status());
                if assist.dialog_open() {
                    render::unwinnable_dialog(&assets);
                }
            }
        }
        next_frame().await;
    }
}

/// Handle input while the unwinnable dialog is open: only its buttons respond.
fn handle_dialog(
    ptr: &Pointer,
    session: &mut Session,
    assist: &mut Assist,
    anim: &mut Animator,
    game_start: &mut f64,
    cfg: GameConfig,
) {
    if !ptr.pressed {
        return;
    }
    for (choice, r) in render::dialog_button_rects() {
        if ptr.x >= r.x && ptr.x <= r.x + r.w && ptr.y >= r.y && ptr.y <= r.y + r.h {
            match choice {
                DialogChoice::Continue => assist.dismiss_dialog(),
                DialogChoice::NewGame => new_game(session, anim, game_start, cfg),
            }
            return;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_input(
    session: &mut Session,
    drag: &mut Option<Drag>,
    anim: &mut Animator,
    layout: &Layout,
    cfg: GameConfig,
    game_start: &mut f64,
    last_tap: &mut f64,
    last_hit: &mut Option<Hit>,
    ptr: &Pointer,
) {
    // Keyboard commands (native) stay available alongside touch controls.
    if is_key_pressed(KeyCode::N) {
        new_game(session, anim, game_start, cfg);
        *drag = None;
    }
    if is_key_pressed(KeyCode::U) {
        session.undo();
        *drag = None;
    }
    if is_key_pressed(KeyCode::R) {
        session.redo();
        *drag = None;
    }
    if is_key_pressed(KeyCode::Escape) {
        *drag = None;
    }
    if is_key_pressed(KeyCode::Enter) && session.state.waste.top().is_some() {
        auto_move(session, anim, layout, Source::Waste);
    }

    if ptr.pressed && drag.is_none() {
        // On-screen control buttons take priority over the board.
        if let Some(btn) = layout.button_at(ptr.x, ptr.y) {
            match btn {
                ButtonId::Undo => session.undo(),
                ButtonId::New => new_game(session, anim, game_start, cfg),
            }
            return;
        }

        let hit = hit_test(&session.state, layout, ptr.x, ptr.y);
        let now = get_time();
        let double = now - *last_tap < 0.35 && *last_hit == Some(hit);
        *last_tap = now;
        *last_hit = Some(hit);

        if double {
            if let Some(s) = source_of(&session.state, hit) {
                auto_move(session, anim, layout, s);
            }
        } else if hit == Hit::Stock {
            let mv = if session.state.stock.is_empty() {
                Move::Recycle
            } else {
                Move::Draw
            };
            session.apply(mv);
        } else if let Some(src) = source_of(&session.state, hit) {
            let (origin, cards) = grab_run(src, layout, &session.state);
            *drag = Some(Drag {
                source: src,
                cards,
                grab_dx: ptr.x - origin.x,
                grab_dy: ptr.y - origin.y,
                pos: vec2(ptr.x, ptr.y),
                origin,
            });
        }
    } else if let Some(d) = drag.as_mut() {
        d.pos = vec2(ptr.x, ptr.y);
        if ptr.released {
            let d = drag.take().unwrap();
            resolve_drop(session, anim, layout, d);
        }
    }
}

/// Start a fresh game, clearing any in-flight animations.
fn new_game(session: &mut Session, anim: &mut Animator, game_start: &mut f64, cfg: GameConfig) {
    *session = Session::new(random_seed(), cfg);
    *game_start = get_time();
    anim.anims.clear();
}

/// The top-left of a source's grabbed card and the run it carries.
fn grab_run(source: Source, layout: &Layout, state: &klondike::GameState) -> (Vec2, Vec<Card>) {
    match source {
        Source::TableauRun { col, index } => {
            let r = layout.tableau_card_rect(col, index);
            (vec2(r.x, r.y), state.tableau[col].cards()[index..].to_vec())
        }
        Source::Waste => {
            let shown = state.waste.len().min(3);
            let x = layout.waste.x + shown.saturating_sub(1) as f32 * layout.card_w * 0.28;
            (
                vec2(x, layout.waste.y),
                state.waste.top().into_iter().collect(),
            )
        }
        Source::Foundation(i) => {
            let r = layout.foundations[i];
            (
                vec2(r.x, r.y),
                state.foundations[i].top().into_iter().collect(),
            )
        }
    }
}

/// Resting top-left where a `count`-card group lands on `pile` (after apply).
fn resting_top_left(
    pile: Pile,
    count: usize,
    layout: &Layout,
    state: &klondike::GameState,
) -> Vec2 {
    match pile {
        Pile::Foundation(k) => vec2(layout.foundations[k].x, layout.foundations[k].y),
        Pile::Tableau(k) => {
            let start = state.tableau[k].len().saturating_sub(count);
            let r = layout.tableau_card_rect(k, start);
            vec2(r.x, r.y)
        }
        Pile::Waste => vec2(layout.waste.x, layout.waste.y),
    }
}

/// The pile a source occupies (for suppressing its cards during a return snap).
fn source_pile(source: Source, count: usize) -> (Pile, usize) {
    match source {
        Source::TableauRun { col, .. } => (Pile::Tableau(col), count),
        Source::Waste => (Pile::Waste, 1),
        Source::Foundation(i) => (Pile::Foundation(i), 1),
    }
}

/// Apply `mv`, then enqueue a snap animation of `cards` from `from` to the
/// pile's resting position (state changes first; animation is cosmetic).
fn animate_move(
    session: &mut Session,
    anim: &mut Animator,
    layout: &Layout,
    mv: Move,
    cards: Vec<Card>,
    from: Vec2,
) -> bool {
    if !session.apply(mv) {
        return false;
    }
    if let Some((dest, count)) = move_dest(&mv) {
        let to = resting_top_left(dest, count, layout, &session.state);
        anim.push(snap(cards, from, to, layout, Some((dest, count))));
    }
    true
}

/// Build a snap animation for `cards` traveling `from` → `to`.
fn snap(
    cards: Vec<Card>,
    from: Vec2,
    to: Vec2,
    layout: &Layout,
    hide: Option<(Pile, usize)>,
) -> CardAnim {
    CardAnim {
        cards,
        from,
        to,
        fan_dy: layout.fan_dy,
        card_w: layout.card_w,
        start: get_time(),
        dur: SNAP_SECS,
        hide,
    }
}

/// Resolve a released drag: snap into the nearest legal pile, else back to origin.
fn resolve_drop(session: &mut Session, anim: &mut Animator, layout: &Layout, d: Drag) {
    let from = d.top_left();
    if let Some(pile) = nearest_pile(&session.state, layout, d.pos.x, d.pos.y) {
        if let Some(mv) = resolve(&session.state, d.source, pile) {
            animate_move(session, anim, layout, mv, d.cards, from);
            return;
        }
    }
    // A tap (released ~where it was pressed) is a no-op: no move, no message, so
    // it doesn't fight the double-tap auto-move. A real drag to nowhere returns
    // to origin with brief feedback.
    let moved = (from - d.origin).length() > layout.card_w * 0.25;
    if !moved {
        return;
    }
    session.set_message("No move");
    let (src_pile, count) = source_pile(d.source, d.cards.len());
    anim.push(snap(
        d.cards,
        from,
        d.origin,
        layout,
        Some((src_pile, count)),
    ));
}

/// Auto-move a source to its best legal destination (double-tap / Enter).
fn auto_move(session: &mut Session, anim: &mut Animator, layout: &Layout, source: Source) {
    match auto_target(&session.state, source) {
        Some(mv) => {
            let (origin, cards) = grab_run(source, layout, &session.state);
            animate_move(session, anim, layout, mv, cards, origin);
        }
        None => session.set_message("No move for that card"),
    }
}

/// Apply and animate a queued playback move (source inferred from the move).
fn play_queued(session: &mut Session, anim: &mut Animator, layout: &Layout, mv: Move) {
    let source = match mv {
        Move::WasteToFoundation { .. } | Move::WasteToTableau { .. } => Some(Source::Waste),
        Move::FoundationToTableau { foundation, .. } => Some(Source::Foundation(foundation)),
        Move::TableauToFoundation { column, .. } => Some(Source::TableauRun {
            col: column,
            index: session.state.tableau[column].len().saturating_sub(1),
        }),
        Move::TableauToTableau { from, count, .. } => Some(Source::TableauRun {
            col: from,
            index: session.state.tableau[from].len().saturating_sub(count),
        }),
        Move::Draw | Move::Recycle => None,
    };
    match source {
        Some(src) => {
            let (from, cards) = grab_run(src, layout, &session.state);
            animate_move(session, anim, layout, mv, cards, from);
        }
        None => {
            session.apply(mv);
        }
    }
}
