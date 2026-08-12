//! Terminal rendering of the board, using color and Unicode. This is the only
//! module that writes to the terminal.

use std::io::{self, Write};

use crossterm::style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::{cursor, queue, terminal};

use crate::cli::session::Session;
use crate::cli::Pile;
use klondike::{Card, Color as CardColor};

const CELL_W: usize = 5;
/// Foundation keys, in display order (foundations 0..4).
const FOUNDATION_KEYS: [char; 4] = ['8', '9', '0', '-'];

/// Draw the whole screen for the current session state.
pub fn render<W: Write>(w: &mut W, s: &Session) -> io::Result<()> {
    queue!(
        w,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    if s.help_visible() {
        return render_help(w);
    }

    // Title + status line.
    line(w, 0, |w| {
        queue!(
            w,
            SetAttribute(Attribute::Bold),
            Print("♠ Klondike Solitaire ♥"),
            SetAttribute(Attribute::Reset),
        )
    })?;
    let score = s.state().current_score();
    line(w, 1, |w| {
        queue!(
            w,
            Print(format!(
                "seed {}   moves {}   score {}   time {}",
                s.seed(),
                s.move_count(),
                score,
                fmt_time(s.elapsed_secs()),
            ))
        )
    })?;

    // Stock / waste / foundations. Fixed column anchors keep each foundation
    // cell aligned exactly under its [8][9][0][-] key header (and stock/waste
    // under theirs), regardless of how many waste cards are shown.
    let sel = s.selection();
    let cell_w = CELL_W as u16;
    let stock_x: u16 = 1;
    let waste_x: u16 = stock_x + cell_w + 1;
    // Foundations sit past the 4-wide waste region and a 2-column gap.
    let fnd_x: u16 = waste_x + 4 * cell_w + 2;

    // Header row.
    queue!(w, cursor::MoveTo(stock_x, 3), Print("Stock"))?;
    queue!(w, cursor::MoveTo(waste_x, 3), Print("Waste"))?;
    for (i, k) in FOUNDATION_KEYS.iter().enumerate() {
        queue!(w, cursor::MoveTo(fnd_x + i as u16 * cell_w, 3))?;
        label(w, &format!("[{k}]"), sel == Some(Pile::Foundation(i)))?;
    }

    // Cell row: stock (highlighted when selected — empty stock shows `( )` to
    // signal "space again to recycle"), then the waste's last few, then each
    // foundation under its header.
    let stock_selected = sel == Some(Pile::StockWaste);
    let stock_glyph = if s.state().stock.is_empty() {
        "( )"
    } else {
        "###"
    };
    queue!(w, cursor::MoveTo(stock_x, 4))?;
    if stock_selected {
        highlight_cell(w, stock_glyph)?;
    } else {
        dim_cell(w, stock_glyph)?;
    }

    queue!(w, cursor::MoveTo(waste_x, 4))?;
    let waste = s.state().waste.cards();
    if waste.is_empty() {
        empty_cell(w)?;
    } else {
        let start = waste.len().saturating_sub(3);
        for c in &waste[start..] {
            card_cell(w, *c)?;
        }
    }

    for (i, f) in s.state().foundations.iter().enumerate() {
        queue!(w, cursor::MoveTo(fnd_x + i as u16 * cell_w, 4))?;
        match f.top() {
            Some(c) => card_cell(w, c)?,
            None => empty_cell(w)?,
        }
    }

    // Tableau header (column keys 1..7).
    line(w, 6, |w| {
        queue!(w, Print(" "))?;
        for c in 0..7 {
            label(w, &format!("[{}]", c + 1), sel == Some(Pile::Tableau(c)))?;
        }
        Ok(())
    })?;

    // Tableau grid.
    let cols = &s.state().tableau;
    let height = cols.iter().map(|c| c.len()).max().unwrap_or(0);
    for row in 0..height.max(1) {
        queue!(w, cursor::MoveTo(0, 7 + row as u16), Print(" "))?;
        for col in cols.iter() {
            match col.cards().get(row) {
                Some(card) => card_cell(w, *card)?,
                None => {
                    if row == 0 && col.is_empty() {
                        empty_cell(w)?;
                    } else {
                        queue!(w, Print(" ".repeat(CELL_W)))?;
                    }
                }
            }
        }
    }

    // Footer: message + hint.
    let footer = 8 + height.max(1) as u16;
    if s.is_won() {
        line(w, footer, |w| {
            queue!(
                w,
                SetForegroundColor(Color::Green),
                SetAttribute(Attribute::Bold),
                Print(format!(
                    "🎉 You win!  final score {}  in {}  —  n: new game   q: quit",
                    s.state().final_score(),
                    fmt_time(s.elapsed_secs()),
                )),
                SetAttribute(Attribute::Reset),
                ResetColor,
            )
        })?;
    } else if let Some(msg) = s.message() {
        line(w, footer, |w| {
            queue!(w, SetForegroundColor(Color::Yellow), Print(msg), ResetColor)
        })?;
    }
    line(w, footer + 1, |w| {
        queue!(
            w,
            SetForegroundColor(Color::DarkGrey),
            Print("? help   space deal/select   Enter auto   u undo   r redo   n new   q quit"),
            ResetColor,
        )
    })?;

    w.flush()
}

/// The compact help/legend overlay.
fn render_help<W: Write>(w: &mut W) -> io::Result<()> {
    let lines = [
        "Klondike — key legend",
        "",
        "  1-7            select a tableau column",
        "  8 9 0 -        select a foundation",
        "  space          deal from stock (recycle when empty);",
        "                 with a waste card, selects it — space again deals",
        "  <src><dst>     move: e.g. 2 5  (col2 -> col5),  1 8  (col1 -> foundation)",
        "  space <dst>    move the waste's top card",
        "  Enter          auto-play a card to its best legal spot",
        "  Esc            cancel a pending selection",
        "",
        "  u undo   r redo   n new game   q quit   ? close help",
        "",
        "Illegal moves are never applied. Foundations auto-route by suit.",
    ];
    for (i, text) in lines.iter().enumerate() {
        queue!(w, cursor::MoveTo(2, 1 + i as u16), Print(*text))?;
    }
    w.flush()
}

// --- small drawing helpers ---

/// Move to the start of row `y`, run `f`, used for whole-line segments.
fn line<W: Write>(w: &mut W, y: u16, f: impl FnOnce(&mut W) -> io::Result<()>) -> io::Result<()> {
    queue!(w, cursor::MoveTo(0, y))?;
    f(w)
}

/// A pile label like `[3]`, reverse-highlighted when selected, padded to a cell.
fn label<W: Write>(w: &mut W, text: &str, selected: bool) -> io::Result<()> {
    let padded = format!("{text:<width$}", width = CELL_W);
    if selected {
        queue!(
            w,
            SetAttribute(Attribute::Reverse),
            Print(padded),
            SetAttribute(Attribute::Reset)
        )
    } else {
        queue!(w, Print(padded))
    }
}

/// A face-up/face-down card rendered in a fixed-width cell with suit color.
fn card_cell<W: Write>(w: &mut W, card: Card) -> io::Result<()> {
    if !card.face_up {
        return dim_cell(w, "###");
    }
    let text = format!("{}{}", card.rank.label(), card.suit.symbol());
    let color = match card.color() {
        CardColor::Red => Color::Red,
        CardColor::Black => Color::White,
    };
    queue!(
        w,
        SetForegroundColor(color),
        Print(format!("{text:<width$}", width = CELL_W)),
        ResetColor
    )
}

fn dim_cell<W: Write>(w: &mut W, text: &str) -> io::Result<()> {
    queue!(
        w,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("{text:<width$}", width = CELL_W)),
        ResetColor
    )
}

fn highlight_cell<W: Write>(w: &mut W, text: &str) -> io::Result<()> {
    queue!(
        w,
        SetAttribute(Attribute::Reverse),
        Print(format!("{text:<width$}", width = CELL_W)),
        SetAttribute(Attribute::Reset)
    )
}

fn empty_cell<W: Write>(w: &mut W) -> io::Result<()> {
    dim_cell(w, " · ")
}

/// Format seconds as `mm:ss`.
fn fmt_time(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::fmt_time;

    #[test]
    fn time_formatting() {
        assert_eq!(fmt_time(0), "00:00");
        assert_eq!(fmt_time(9), "00:09");
        assert_eq!(fmt_time(83), "01:23");
        assert_eq!(fmt_time(600), "10:00");
    }
}
