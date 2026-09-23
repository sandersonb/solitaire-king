//! Best-effort local persistence for the GUI.
//!
//! Web (wasm): the browser's Web Storage via `quad-storage`. Native: one file per
//! key under the user's config dir (`$XDG_CONFIG_HOME`, else `$HOME/.config`, else
//! `%APPDATA%`), in a `klondike-solitaire/` subdir. Everything is best-effort —
//! absent or unreadable data reads as `None`/defaults and saves never panic, so
//! persistence is never on the critical path of play.

/// App subdirectory for native storage files.
#[cfg(not(target_arch = "wasm32"))]
const APP_DIR: &str = "klondike-solitaire";

// ---- Generic key/value backend -------------------------------------------

#[cfg(target_arch = "wasm32")]
pub fn load(key: &str) -> Option<String> {
    let storage = quad_storage::STORAGE.lock().ok()?;
    storage.get(key)
}

#[cfg(target_arch = "wasm32")]
pub fn save(key: &str, value: &str) {
    if let Ok(mut storage) = quad_storage::STORAGE.lock() {
        storage.set(key, value);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn config_path(key: &str) -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))?;
    Some(base.join(APP_DIR).join(key))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load(key: &str) -> Option<String> {
    std::fs::read_to_string(config_path(key)?).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(key: &str, value: &str) {
    if let Some(path) = config_path(key) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, value);
    }
}

// ---- Typed records ---------------------------------------------------------

const HIGH_SCORE_KEY: &str = "highscore.v1";
const COUNTERS_KEY: &str = "stats.v1";

/// The single best result: highest final score, that game's time, and when it was
/// set (unix seconds).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HighScore {
    pub score: i64,
    pub time_secs: u64,
    pub achieved_at: i64,
}

impl HighScore {
    fn to_field(self) -> String {
        format!("{}|{}|{}", self.score, self.time_secs, self.achieved_at)
    }

    fn parse(s: &str) -> Option<HighScore> {
        let mut it = s.split('|');
        let score = it.next()?.trim().parse().ok()?;
        let time_secs = it.next()?.trim().parse().ok()?;
        let achieved_at = it.next()?.trim().parse().ok()?;
        Some(HighScore {
            score,
            time_secs,
            achieved_at,
        })
    }
}

/// Lifetime play counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub wins: u64,
    pub new_games: u64,
}

impl Counters {
    fn to_field(self) -> String {
        format!("{}|{}", self.wins, self.new_games)
    }

    /// Tolerant parse: any missing/garbled field defaults to 0.
    fn parse(s: &str) -> Counters {
        let mut it = s.split('|');
        let wins = it.next().and_then(|x| x.trim().parse().ok()).unwrap_or(0);
        let new_games = it.next().and_then(|x| x.trim().parse().ok()).unwrap_or(0);
        Counters { wins, new_games }
    }
}

pub fn load_high_score() -> Option<HighScore> {
    HighScore::parse(&load(HIGH_SCORE_KEY)?)
}

pub fn save_high_score(h: &HighScore) {
    save(HIGH_SCORE_KEY, &h.to_field());
}

pub fn load_counters() -> Counters {
    load(COUNTERS_KEY)
        .map(|s| Counters::parse(&s))
        .unwrap_or_default()
}

pub fn save_counters(c: &Counters) {
    save(COUNTERS_KEY, &c.to_field());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_score_round_trips() {
        let h = HighScore {
            score: 1234,
            time_secs: 87,
            achieved_at: 1_700_000_000,
        };
        assert_eq!(HighScore::parse(&h.to_field()), Some(h));
        // Negative scores are valid.
        let n = HighScore {
            score: -52,
            time_secs: 0,
            achieved_at: 1,
        };
        assert_eq!(HighScore::parse(&n.to_field()), Some(n));
    }

    #[test]
    fn malformed_high_score_is_none() {
        assert_eq!(HighScore::parse(""), None);
        assert_eq!(HighScore::parse("1|2"), None);
        assert_eq!(HighScore::parse("x|2|3"), None);
        assert_eq!(HighScore::parse("1|two|3"), None);
    }

    #[test]
    fn counters_round_trip_and_tolerant() {
        let c = Counters {
            wins: 7,
            new_games: 42,
        };
        assert_eq!(Counters::parse(&c.to_field()), c);
        // Malformed/empty fall back to zero without error.
        assert_eq!(Counters::parse(""), Counters::default());
        assert_eq!(Counters::parse("9"), Counters { wins: 9, new_games: 0 });
        assert_eq!(Counters::parse("bad|data"), Counters::default());
    }
}
