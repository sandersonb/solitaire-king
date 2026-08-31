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
use render::{DialogChoice, SettingRow, SolverAction};
use session::Session;
use solver::{Assist, Status};

/// Press-and-hold threshold for the Undo button to redo (seconds).
const UNDO_HOLD_SECS: f64 = 0.4;

/// In-session settings (not persisted across launches).
struct Settings {
    draw_three: bool,
    solver_enabled: bool,
    show_seed: bool,
}

impl Settings {
    fn from_config(cfg: GameConfig) -> Self {
        Settings {
            draw_three: matches!(cfg.draw_mode, DrawMode::Three),
            solver_enabled: true,
            show_seed: true,
        }
    }

    /// The game config for a new deal, applying the chosen draw mode.
    fn game_config(&self, base: GameConfig) -> GameConfig {
        GameConfig {
            draw_mode: if self.draw_three {
                DrawMode::Three
            } else {
                DrawMode::One
            },
            ..base
        }
    }
}

/// Which modal overlay (if any) is open. The unwinnable dialog is tracked
/// separately by the assist.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    Solver,
    Settings,
}

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
    let mut settings = Settings::from_config(cfg);
    let mut overlay = Overlay::None;
    let mut undo_press: Option<f64> = None;
    let mut undo_fired = false;
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
                // The timer is frozen at zero while auto-solving (not a scored win).
                if session.is_auto_solving() {
                    session.set_elapsed(0);
                } else if !session.is_won() {
                    session.set_elapsed((get_time() - game_start).max(0.0) as u64);
                }
                anim.tick(get_time());

                // Input priority: unwinnable dialog → overlay → auto-solve → board.
                if assist.dialog_open() {
                    handle_dialog(
                        &ptr,
                        &mut session,
                        &mut assist,
                        &mut anim,
                        &mut game_start,
                        &settings,
                        cfg,
                    );
                } else if overlay == Overlay::Solver {
                    handle_solver_overlay(
                        &ptr,
                        &mut session,
                        &mut assist,
                        &mut anim,
                        &mut game_start,
                        &settings,
                        cfg,
                        &mut overlay,
                    );
                } else if overlay == Overlay::Settings {
                    handle_settings(&ptr, &mut assist, &mut settings, &session, &mut overlay);
                } else if session.is_auto_solving() {
                    handle_autosolve(&ptr, &mut session, &mut anim);
                } else {
                    handle_input(
                        &mut session,
                        &mut drag,
                        &mut anim,
                        &mut assist,
                        &layout,
                        &settings,
                        cfg,
                        &mut game_start,
                        &mut overlay,
                        &mut last_tap,
                        &mut last_hit,
                        &mut undo_press,
                        &mut undo_fired,
                        &ptr,
                    );
                }

                // Drive auto-solve playback at its cadence.
                if session.is_auto_solving() {
                    if drag.is_none() && !anim.is_animating() {
                        if let Some(mv) = anim.take_next(get_time()) {
                            play_queued(&mut session, &mut anim, &layout, mv);
                        } else if !anim.has_queued() {
                            if session.is_won() {
                                session.finish_auto_solve();
                            } else {
                                session.cancel_auto_solve();
                            }
                        }
                    }
                } else {
                    // Detect state changes so the assist re-evaluates.
                    if session.seed() != last_seed {
                        assist.reset(&session.state);
                    } else if klondike::encode(&session.state) != last_key {
                        assist.on_state_change(&session.state);
                    }
                    if ptr.pressed || ptr.released {
                        assist.note_activity();
                    }
                    assist.update(&session.state);
                }
                // Keep the seed/key baseline current (also across auto-solve).
                last_seed = session.seed();
                last_key = klondike::encode(&session.state);

                render::board(
                    &session,
                    &assets,
                    &layout,
                    drag.as_ref(),
                    &anim,
                    settings.show_seed,
                );
                render::solver_indicator(&assets, &layout, assist.status(), !assist.enabled());
                if assist.dialog_open() {
                    render::unwinnable_dialog(&assets);
                } else if overlay == Overlay::Solver {
                    render::solver_overlay(
                        &assets,
                        assist.status(),
                        assist.solution_len(&session.state),
                        assist.enabled(),
                    );
                } else if overlay == Overlay::Settings {
                    render::settings_overlay(
                        &assets,
                        settings.draw_three,
                        settings.solver_enabled,
                        settings.show_seed,
                    );
                }
            }
        }
        next_frame().await;
    }
}

/// Handle input while the unwinnable dialog is open: only its buttons respond.
#[allow(clippy::too_many_arguments)]
fn handle_dialog(
    ptr: &Pointer,
    session: &mut Session,
    assist: &mut Assist,
    anim: &mut Animator,
    game_start: &mut f64,
    settings: &Settings,
    cfg: GameConfig,
) {
    if !ptr.pressed {
        return;
    }
    for (choice, r) in render::dialog_button_rects() {
        if rect_hit(r, ptr) {
            match choice {
                DialogChoice::Continue => assist.dismiss_dialog(),
                DialogChoice::NewGame => {
                    new_game(session, anim, game_start, settings.game_config(cfg))
                }
            }
            return;
        }
    }
}

/// Handle input for the solver status overlay.
#[allow(clippy::too_many_arguments)]
fn handle_solver_overlay(
    ptr: &Pointer,
    session: &mut Session,
    assist: &mut Assist,
    anim: &mut Animator,
    game_start: &mut f64,
    settings: &Settings,
    cfg: GameConfig,
    overlay: &mut Overlay,
) {
    if !ptr.pressed {
        return;
    }
    for (action, r) in render::solver_overlay_actions(
        assist.status(),
        assist.solution_len(&session.state).is_some(),
        assist.enabled(),
    ) {
        if rect_hit(r, ptr) {
            match action {
                SolverAction::AutoSolve => start_autosolve(session, assist, anim),
                SolverAction::NewGame => {
                    new_game(session, anim, game_start, settings.game_config(cfg))
                }
                SolverAction::Close => {}
            }
            *overlay = Overlay::None;
            return;
        }
    }
    // Clicking outside the panel closes it.
    *overlay = Overlay::None;
}

/// Handle input for the Settings dialog.
fn handle_settings(
    ptr: &Pointer,
    assist: &mut Assist,
    settings: &mut Settings,
    session: &Session,
    overlay: &mut Overlay,
) {
    if !ptr.pressed {
        return;
    }
    for (row, r) in render::settings_rows() {
        if rect_hit(r, ptr) {
            match row {
                SettingRow::DrawMode => settings.draw_three = !settings.draw_three,
                SettingRow::Solver => {
                    settings.solver_enabled = !settings.solver_enabled;
                    assist.set_enabled(settings.solver_enabled, &session.state);
                }
                SettingRow::Seed => settings.show_seed = !settings.show_seed,
                SettingRow::Close => *overlay = Overlay::None,
            }
            return;
        }
    }
    *overlay = Overlay::None; // click outside closes
}

/// While auto-solving, any press or key cancels playback and returns to play.
fn handle_autosolve(ptr: &Pointer, session: &mut Session, anim: &mut Animator) {
    if ptr.pressed || get_last_key_pressed().is_some() {
        anim.clear_queue();
        session.cancel_auto_solve();
    }
}

/// Begin auto-solving if a winning line is known for the current position.
fn start_autosolve(session: &mut Session, assist: &Assist, anim: &mut Animator) {
    if let Some(moves) = assist.solution_for(&session.state) {
        let moves = moves.to_vec();
        session.begin_auto_solve();
        anim.enqueue_moves(&moves, get_time());
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_input(
    session: &mut Session,
    drag: &mut Option<Drag>,
    anim: &mut Animator,
    assist: &mut Assist,
    layout: &Layout,
    settings: &Settings,
    cfg: GameConfig,
    game_start: &mut f64,
    overlay: &mut Overlay,
    last_tap: &mut f64,
    last_hit: &mut Option<Hit>,
    undo_press: &mut Option<f64>,
    undo_fired: &mut bool,
    ptr: &Pointer,
) {
    // Keyboard commands stay available (no longer advertised on screen).
    if is_key_pressed(KeyCode::N) {
        new_game(session, anim, game_start, settings.game_config(cfg));
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
    // Shift+A auto-solves when a solution is known.
    if is_key_pressed(KeyCode::A)
        && (is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift))
        && assist.solution_for(&session.state).is_some()
    {
        start_autosolve(session, assist, anim);
        return;
    }

    // Undo/Redo button: tap = undo, press-and-hold = one redo.
    let over_undo = layout.button_at(ptr.x, ptr.y) == Some(ButtonId::UndoRedo);
    if ptr.pressed && over_undo && drag.is_none() {
        *undo_press = Some(get_time());
        *undo_fired = false;
        return;
    }
    if undo_press.is_some() {
        let start = undo_press.unwrap();
        if ptr.down && over_undo {
            if !*undo_fired && get_time() - start > UNDO_HOLD_SECS {
                session.redo();
                *undo_fired = true;
            }
        } else {
            if ptr.released && over_undo && !*undo_fired {
                session.undo();
            }
            *undo_press = None;
        }
        return;
    }

    if ptr.pressed && drag.is_none() {
        // Other control buttons and the indicator take priority over the board.
        if let Some(btn) = layout.button_at(ptr.x, ptr.y) {
            match btn {
                ButtonId::New => new_game(session, anim, game_start, settings.game_config(cfg)),
                ButtonId::Settings => *overlay = Overlay::Settings,
                ButtonId::UndoRedo => {}
            }
            return;
        }
        if layout.indicator_at(ptr.x, ptr.y) {
            if assist.status() != Status::Checking {
                *overlay = Overlay::Solver;
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

/// Whether the pointer is over rect `r`.
fn rect_hit(r: macroquad::prelude::Rect, ptr: &Pointer) -> bool {
    ptr.x >= r.x && ptr.x <= r.x + r.w && ptr.y >= r.y && ptr.y <= r.y + r.h
}

/// Start a fresh game, clearing any in-flight animations.
fn new_game(session: &mut Session, anim: &mut Animator, game_start: &mut f64, cfg: GameConfig) {
    *session = Session::new(random_seed(), cfg);
    *game_start = get_time();
    anim.anims.clear();
    anim.clear_queue();
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
