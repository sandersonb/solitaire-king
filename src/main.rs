//! Binary entry point for the Klondike CLI.

mod cli;

use std::io::{self, stdout, Stdout, Write};
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::{cursor, execute, terminal};

use cli::render::render;
use cli::session::{random_seed, Session};
use cli::Signal;
use klondike::{DrawMode, GameConfig};

/// Klondike Solitaire — classic rules, Windows Standard scoring.
#[derive(Parser, Debug)]
#[command(name = "klondike", version, about)]
struct Args {
    // --- Game ---
    /// Run the interactive terminal game (default and only mode for now).
    #[arg(long, default_value_t = true, help_heading = "Game")]
    cli: bool,

    /// Deal seed as a memorable proquint string (e.g. lusab-babad-gutih-tugad) or
    /// a raw u64; a random seed is used when omitted.
    #[arg(short, long, help_heading = "Game")]
    seed: Option<String>,

    /// Cards turned per stock draw: 1 or 3.
    #[arg(long, default_value_t = 3, help_heading = "Game")]
    draw: u8,

    /// Enable timed scoring (time penalty + win bonus).
    #[arg(long, default_value_t = false, help_heading = "Game")]
    timed: bool,

    /// Maximum stock recycles; unlimited when omitted.
    #[arg(long, help_heading = "Game")]
    redeal: Option<u32>,

    // --- Solver search ---
    /// Solve the deal with the solver instead of playing it.
    #[arg(long, default_value_t = false, help_heading = "Solver search")]
    solve: bool,

    /// Budget: maximum nodes to expand (0 = unlimited).
    #[arg(long, default_value_t = 10_000_000, help_heading = "Solver search")]
    max_nodes: u64,

    /// Budget: maximum seconds to search (0 = unlimited).
    #[arg(long, default_value_t = 15, help_heading = "Solver search")]
    max_time: u64,

    // --- Solver heuristics ---
    /// Disable all heuristics and the table at once (reproduce the naive baseline).
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    baseline: bool,

    /// Disable safe foundation auto-moves.
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    no_safe_automoves: bool,

    /// Disable heuristic move ordering.
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    no_ordering: bool,

    /// Disable empty-column symmetry pruning.
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    no_symmetry: bool,

    /// Prefer digging the smaller source column instead of the larger.
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    dig_smaller: bool,

    /// Enable the experimental equivalence pruning (off by default).
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    equivalence: bool,

    /// Disable no-op pruning (for experiments).
    #[arg(long, default_value_t = false, help_heading = "Solver heuristics")]
    no_noop: bool,

    // --- Transposition table ---
    /// Disable the global transposition table (use per-path search only).
    #[arg(long, default_value_t = false, help_heading = "Transposition table")]
    no_transposition: bool,

    /// Transposition-table capacity in entries (0 = default).
    #[arg(long, default_value_t = 0, help_heading = "Transposition table")]
    max_table_entries: usize,

    /// Use exact byte keys instead of the default Zobrist hash (collision-free reference).
    #[arg(long, default_value_t = false, help_heading = "Transposition table")]
    exact_keys: bool,
}

/// Translate CLI args into a `GameConfig`, validating enumerated values.
fn build_config(draw: u8, timed: bool, redeal: Option<u32>) -> Result<GameConfig, String> {
    let draw_mode = match draw {
        1 => DrawMode::One,
        3 => DrawMode::Three,
        other => return Err(format!("--draw must be 1 or 3 (got {other})")),
    };
    Ok(GameConfig {
        draw_mode,
        redeal_limit: redeal,
        timed,
    })
}

fn main() {
    let args = Args::parse();
    let _ = args.cli; // Only the CLI mode exists today; kept for forward compatibility.

    let config = match build_config(args.draw, args.timed, args.redeal) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}\n\nRun with --help for usage.");
            std::process::exit(2);
        }
    };
    let seed = match args.seed {
        Some(ref s) => match klondike::seed::decode(s) {
            Some(v) => v,
            None => {
                eprintln!(
                    "error: --seed must be a proquint string or a u64 (got {s:?})\n\n\
                     Run with --help for usage."
                );
                std::process::exit(2);
            }
        },
        None => random_seed(),
    };

    // Solver mode: run the brute-force solver and print a report, no terminal.
    if args.solve {
        let budget = cli::solve::build_budget(args.max_nodes, args.max_time);
        let options = cli::solve::build_options(cli::solve::HeuristicFlags {
            equivalence: args.equivalence,
            no_noop: args.no_noop,
            baseline: args.baseline,
            no_safe_automoves: args.no_safe_automoves,
            no_ordering: args.no_ordering,
            no_symmetry: args.no_symmetry,
            dig_smaller: args.dig_smaller,
            no_transposition: args.no_transposition,
            max_table_entries: args.max_table_entries,
            exact_keys: args.exact_keys,
        });
        cli::solve::run(seed, config, budget, options);
        return;
    }

    let mut session = Session::new(seed, config);

    // The terminal guard restores the screen on scope exit, including panics.
    let result = {
        let _guard = match TerminalGuard::new() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("error: could not set up terminal: {e}");
                std::process::exit(1);
            }
        };
        run_loop(&mut session)
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }

    print_summary(&mut session);
}

/// The render → input → apply loop. Polls so the clock display keeps ticking
/// even without keypresses.
fn run_loop(session: &mut Session) -> io::Result<()> {
    let mut out = stdout();
    loop {
        session.sync_time();
        render(&mut out, session)?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                // Ctrl-C quits cleanly (raw mode swallows the default signal).
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                if let Some(input) = translate(key.code) {
                    if session.handle_key(input) == Signal::Quit {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

fn translate(code: KeyCode) -> Option<cli::session::KeyInput> {
    use cli::session::KeyInput;
    match code {
        KeyCode::Char(c) => Some(KeyInput::Char(c)),
        KeyCode::Enter => Some(KeyInput::Enter),
        KeyCode::Esc => Some(KeyInput::Esc),
        _ => None,
    }
}

/// Print a session summary (seed, result, move history) on the normal screen
/// after the terminal has been restored.
fn print_summary(session: &mut Session) {
    println!("Klondike session over.");
    println!("  seed:  {}", klondike::seed::encode(session.seed()));
    println!("  moves: {}", session.move_count());
    if session.is_won() {
        println!("  result: WON — final score {}", session.final_score());
    } else {
        println!("  result: quit — score {}", session.final_score());
    }
    if !session.history().is_empty() {
        println!("  move history:");
        for (i, mv) in session.history().iter().enumerate() {
            println!("    {:>3}. {:?}", i + 1, mv);
        }
    }
}

/// RAII guard: enters raw mode + alternate screen, and always restores the
/// terminal when dropped (normal exit or panic unwind).
struct TerminalGuard {
    out: Stdout,
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
        out.flush()?;
        Ok(TerminalGuard { out })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(self.out, cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        let _ = self.out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_config_draw_modes() {
        assert_eq!(
            build_config(3, false, None).unwrap().draw_mode,
            DrawMode::Three
        );
        assert_eq!(
            build_config(1, false, None).unwrap().draw_mode,
            DrawMode::One
        );
    }

    #[test]
    fn build_config_rejects_bad_draw() {
        assert!(build_config(2, false, None).is_err());
        assert!(build_config(0, false, None).is_err());
    }

    #[test]
    fn build_config_passes_flags() {
        let c = build_config(1, true, Some(2)).unwrap();
        assert!(c.timed);
        assert_eq!(c.redeal_limit, Some(2));
    }

    #[test]
    fn args_parse_smoke() {
        let a = Args::parse_from(["klondike", "--seed", "42", "--draw", "1", "--timed"]);
        assert_eq!(a.seed.as_deref(), Some("42"));
        assert_eq!(klondike::seed::decode(a.seed.as_deref().unwrap()), Some(42));
        assert_eq!(a.draw, 1);
        assert!(a.timed);

        // A proquint seed argument resolves to the same u64 it encodes.
        let p = klondike::seed::encode(2024);
        let b = Args::parse_from(["klondike", "--seed", &p]);
        assert_eq!(
            klondike::seed::decode(b.seed.as_deref().unwrap()),
            Some(2024)
        );
    }
}
