//! Klondike Solitaire — graphical front-end (macroquad; native + WASM).

mod assets;
mod input;
mod layout;
mod render;
mod session;

use macroquad::prelude::*;

use assets::Assets;
use input::{auto_target, hit_test, pile_of, resolve, source_of, Hit, Source};
use klondike::{DrawMode, GameConfig, Move};
use layout::Layout;
use session::Session;

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
                seed = args.get(i).and_then(|s| s.parse().ok());
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
    let mut selection: Option<Source> = None;
    let mut last_click = 0.0f64;
    let mut last_hit: Option<Hit> = None;

    loop {
        match app {
            App::Splash(since) => {
                render::splash(&assets);
                let shown = get_time() - since;
                let dismiss = is_mouse_button_pressed(MouseButton::Left)
                    || get_last_key_pressed().is_some()
                    || shown > 3.0;
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
                let layout = Layout::compute(screen_width(), screen_height(), max_col_len);
                if !session.is_won() {
                    session.set_elapsed((get_time() - game_start).max(0.0) as u64);
                }
                handle_input(
                    &mut session,
                    &mut selection,
                    &layout,
                    cfg,
                    &mut game_start,
                    &mut last_click,
                    &mut last_hit,
                );
                render::board(&session, &assets, &layout, selection);
            }
        }
        next_frame().await;
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_input(
    session: &mut Session,
    selection: &mut Option<Source>,
    layout: &Layout,
    cfg: GameConfig,
    game_start: &mut f64,
    last_click: &mut f64,
    last_hit: &mut Option<Hit>,
) {
    if is_key_pressed(KeyCode::N) {
        *session = Session::new(random_seed(), cfg);
        *game_start = get_time();
        *selection = None;
    }
    if is_key_pressed(KeyCode::U) {
        session.undo();
        *selection = None;
    }
    if is_key_pressed(KeyCode::R) {
        session.redo();
        *selection = None;
    }
    if is_key_pressed(KeyCode::Escape) {
        *selection = None;
    }
    if is_key_pressed(KeyCode::Enter) {
        let src =
            selection.or_else(|| session.state.waste.top().is_some().then_some(Source::Waste));
        if let Some(s) = src {
            auto_move(session, s);
        }
        *selection = None;
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let hit = hit_test(&session.state, layout, mx, my);
        let now = get_time();
        let double = now - *last_click < 0.35 && *last_hit == Some(hit);
        *last_click = now;
        *last_hit = Some(hit);

        if double {
            if let Some(s) = source_of(&session.state, hit) {
                auto_move(session, s);
            }
            *selection = None;
        } else if hit == Hit::Stock {
            let mv = if session.state.stock.is_empty() {
                Move::Recycle
            } else {
                Move::Draw
            };
            session.apply(mv);
            *selection = None;
        } else if let Some(sel) = *selection {
            if source_of(&session.state, hit) == Some(sel) {
                *selection = None; // clicked the same source: deselect
            } else if let Some(pile) = pile_of(hit) {
                if let Some(mv) = resolve(&session.state, sel, pile) {
                    session.apply(mv);
                    *selection = None;
                } else if let Some(new_src) = source_of(&session.state, hit) {
                    *selection = Some(new_src); // switch selection
                } else {
                    session.set_message("Illegal move");
                    *selection = None;
                }
            } else {
                *selection = None;
            }
        } else {
            *selection = source_of(&session.state, hit);
        }
    }
}

fn auto_move(session: &mut Session, source: Source) {
    match auto_target(&session.state, source) {
        Some(mv) => {
            session.apply(mv);
        }
        None => session.set_message("No move for that card"),
    }
}
