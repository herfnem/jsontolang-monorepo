//! Two-pane terminal UI for jsontolang: JSON on the left, generated types on
//! the right, both scrollable. Renders through `jsontolang_core` directly
//! (same pure path the wasm playground uses) so this needs no mlua/Lua
//! plugins.
//!
//! The left pane is a small vim emulation (Normal/Insert modes, hjkl, gg/G,
//! dd/yy/p/P, u) since the right pane is read-only and only needs navigation.

use anyhow::Result;
use jsontolang_core::{BUILTIN_LANGS, infer_document, render};
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use ratatui_textarea::{CursorMove, TextArea};

const SAMPLE_JSON: &str = "{\n  \"id\": 1,\n  \"name\": \"example\"\n}";
const ROOT_TYPE_NAME: &str = "Root";

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

struct App {
    left: TextArea<'static>,
    right: TextArea<'static>,
    focus: Focus,
    /// Only meaningful while `focus == Focus::Left`; the right pane is
    /// read-only and has no insert mode.
    mode: Mode,
    lang: usize,
    status: String,
    /// Lines captured by the last `yy`/`dd`, pasted by `p`/`P`.
    yank: Vec<String>,
    /// First half of a two-key chord (`gg`, `dd`, `yy`); any other key
    /// cancels it.
    pending: Option<char>,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            left: TextArea::from(SAMPLE_JSON.lines()),
            right: TextArea::default(),
            focus: Focus::Left,
            mode: Mode::Normal,
            lang: 0,
            status: String::new(),
            yank: Vec::new(),
            pending: None,
        };
        app.render_output();
        app
    }

    fn lang_name(&self) -> &'static str {
        BUILTIN_LANGS[self.lang]
    }

    fn left_text(&self) -> String {
        self.left.lines().join("\n")
    }

    fn set_left_text(&mut self, text: &str) {
        self.left = TextArea::from(text.lines());
    }

    /// Re-infers and re-renders the right pane from the left pane's current
    /// text. Cheap enough to call on every keystroke.
    fn render_output(&mut self) {
        let outcome = serde_json::from_str::<serde_json::Value>(&self.left_text())
            .map_err(anyhow::Error::from)
            .and_then(|value| infer_document(ROOT_TYPE_NAME, &value))
            .and_then(|document| render(self.lang_name(), &document));

        let output = match outcome {
            Ok(code) => {
                self.status = format!("rendered {}", self.lang_name());
                code
            }
            Err(error) => {
                self.status = error.to_string();
                format!("// {error}")
            }
        };

        self.right = TextArea::from(output.lines());
        self.refresh_style();
    }

    fn beautify(&mut self) {
        match serde_json::from_str::<serde_json::Value>(&self.left_text()) {
            Ok(value) => {
                self.set_left_text(&serde_json::to_string_pretty(&value).unwrap());
                self.render_output();
            }
            Err(error) => self.status = format!("beautify failed: {error}"),
        }
    }

    fn minify(&mut self) {
        match serde_json::from_str::<serde_json::Value>(&self.left_text()) {
            Ok(value) => {
                self.set_left_text(&serde_json::to_string(&value).unwrap());
                self.render_output();
            }
            Err(error) => self.status = format!("minify failed: {error}"),
        }
    }

    fn copy(&mut self) {
        let text = match self.focus {
            Focus::Left => self.left_text(),
            Focus::Right => self.right.lines().join("\n"),
        };
        let pane = if self.focus == Focus::Left {
            "left"
        } else {
            "right"
        };

        self.status = match set_clipboard_text(text) {
            Ok(()) => format!("copied {pane} pane to clipboard"),
            Err(error) => format!("copy failed: {error}"),
        };
    }

    /// Always targets the left pane, clearing whatever was there first.
    /// Distinct from `p`/`P`, which paste the internal yank register instead
    /// of the system clipboard and don't touch the rest of the buffer.
    fn paste(&mut self) {
        match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
            Ok(text) => {
                self.set_left_text(&text);
                self.render_output();
                self.status = "pasted clipboard into left pane".to_string();
            }
            Err(error) => self.status = format!("paste failed: {error}"),
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Left => Focus::Right,
            Focus::Right => Focus::Left,
        };
        self.mode = Mode::Normal;
        self.pending = None;
        self.refresh_style();
    }

    fn cycle_lang(&mut self) {
        self.lang = (self.lang + 1) % BUILTIN_LANGS.len();
        self.render_output();
    }

    fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
        self.refresh_style();
    }

    fn append(&mut self) {
        self.left.move_cursor(CursorMove::Forward);
        self.enter_insert();
    }

    fn insert_at_line_start(&mut self) {
        self.left.move_cursor(CursorMove::Head);
        self.enter_insert();
    }

    fn insert_at_line_end(&mut self) {
        self.left.move_cursor(CursorMove::End);
        self.enter_insert();
    }

    fn open_below(&mut self) {
        self.left.move_cursor(CursorMove::End);
        self.left.insert_newline();
        self.mode = Mode::Insert;
        self.render_output();
    }

    fn open_above(&mut self) {
        self.left.move_cursor(CursorMove::Head);
        self.left.insert_newline();
        self.left.move_cursor(CursorMove::Up);
        self.mode = Mode::Insert;
        self.render_output();
    }

    fn delete_char_at_cursor(&mut self) {
        if self.left.delete_next_char() {
            self.render_output();
        }
    }

    fn undo(&mut self) {
        if self.left.undo() {
            self.render_output();
        }
    }

    fn yank_line(&mut self) {
        let row = self.left.cursor().0;
        if let Some(line) = self.left.lines().get(row) {
            self.yank = vec![line.clone()];
            self.status = "yanked 1 line".to_string();
        }
    }

    fn delete_line(&mut self) {
        let mut lines = self.left.lines().to_vec();
        if lines.is_empty() {
            return;
        }
        let row = self.left.cursor().0.min(lines.len() - 1);
        self.yank = vec![lines.remove(row)];
        if lines.is_empty() {
            lines.push(String::new());
        }
        let new_row = row.min(lines.len() - 1);
        self.left = TextArea::from(lines);
        self.left.move_cursor(CursorMove::Jump(new_row as u16, 0));
        self.render_output();
        self.status = "deleted 1 line".to_string();
    }

    /// `offset` is 0 for `P` (paste above the cursor line) or 1 for `p`
    /// (paste below).
    fn paste_yank(&mut self, offset: usize) {
        if self.yank.is_empty() {
            self.status = "yank register is empty".to_string();
            return;
        }
        let mut lines = self.left.lines().to_vec();
        let row = self.left.cursor().0;
        let at = (row + offset).min(lines.len());
        let count = self.yank.len();
        for (index, line) in self.yank.iter().enumerate() {
            lines.insert(at + index, line.clone());
        }
        self.left = TextArea::from(lines);
        self.left.move_cursor(CursorMove::Jump(at as u16, 0));
        self.render_output();
        self.status = format!("pasted {count} line(s)");
    }

    fn refresh_style(&mut self) {
        let focused = Style::default().fg(Color::Yellow);
        let unfocused = Style::default().fg(Color::DarkGray);
        let cursor_on = Style::default().add_modifier(Modifier::REVERSED);
        let cursor_off = Style::default();

        let left_focused = self.focus == Focus::Left;
        let mode_label = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        };

        self.left.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if left_focused { focused } else { unfocused })
                .title(format!(" JSON Input [{mode_label}] ")),
        );
        self.left
            .set_cursor_style(if left_focused { cursor_on } else { cursor_off });
        self.left.set_cursor_line_style(Style::default());

        self.right.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if left_focused { unfocused } else { focused })
                .title(format!(" Output: {} ", self.lang_name())),
        );
        self.right
            .set_cursor_style(if left_focused { cursor_off } else { cursor_on });
        self.right.set_cursor_line_style(Style::default());
    }
}

/// X11/Wayland only serve clipboard contents while the owning process is
/// alive, so a `Clipboard` set-and-immediately-dropped on the main thread
/// loses the text before another app can read it. Fix (per arboard's own
/// docs): hand it to a detached thread that blocks in `.wait()` until
/// something else takes ownership of the clipboard, keeping it alive without
/// blocking the UI. Other platforms have no such issue.
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
fn set_clipboard_text(text: String) -> Result<()> {
    use arboard::SetExtLinux;
    std::thread::spawn(move || {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set().wait().text(text);
        }
    });
    Ok(())
}

#[cfg(not(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
)))]
fn set_clipboard_text(text: String) -> Result<()> {
    arboard::Clipboard::new()?.set_text(text)?;
    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    frame.render_widget(&app.left, columns[0]);
    frame.render_widget(&app.right, columns[1]);

    // Basic vim keys (hjkl, gg/G, i/a/I/A/o/O, x, dd, yy, p/P, u) are left
    // off the hint line on purpose — this only spells out the app-specific
    // ones a vim user wouldn't already know.
    let help = match (app.focus, app.mode) {
        (Focus::Left, Mode::Normal) => format!(
            "Tab: focus output | =: beautify | m: minify | Y: copy | \
             R: paste clipboard | L: lang ({}) | Esc: quit | {}",
            app.lang_name(),
            app.status
        ),
        (Focus::Left, Mode::Insert) => format!("Esc: back to normal mode | {}", app.status),
        (Focus::Right, _) => format!(
            "Tab: focus input | L: cycle lang ({}) | Y: copy | Esc: quit | {}",
            app.lang_name(),
            app.status
        ),
    };
    frame.render_widget(Paragraph::new(help), rows[1]);
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        let left_normal = app.focus == Focus::Left && app.mode == Mode::Normal;
        let left_insert = app.focus == Focus::Left && app.mode == Mode::Insert;
        // Plain letters only act as commands outside insert mode — while
        // typing, every key has to reach the buffer as literal text, which
        // is why none of this app's bindings use modifier keys: `navigable`
        // is the only gate they need.
        let navigable = app.focus == Focus::Right || left_normal;

        // Tracks two-key chords (gg/dd/yy): any other key cancels them.
        let chord_key = navigable && matches!(key.code, KeyCode::Char('g' | 'd' | 'y'));
        if !chord_key {
            app.pending = None;
        }

        match key.code {
            KeyCode::Esc if left_insert => {
                app.mode = Mode::Normal;
                app.refresh_style();
            }
            KeyCode::Esc => return Ok(()),
            KeyCode::Tab => app.toggle_focus(),

            _ if left_insert => {
                if app.left.input(Event::Key(key)) {
                    app.render_output();
                }
            }

            // From here on, focus is either the read-only right pane, or the
            // left pane in normal mode: both are navigation, not typing, so
            // plain letters are free to use as commands.
            KeyCode::Char('=') if navigable => app.beautify(),
            KeyCode::Char('m') if navigable => app.minify(),
            KeyCode::Char('Y') if navigable => app.copy(),
            KeyCode::Char('R') if navigable => app.paste(),
            KeyCode::Char('L') if navigable => app.cycle_lang(),
            KeyCode::Char('i') if left_normal => app.enter_insert(),
            KeyCode::Char('a') if left_normal => app.append(),
            KeyCode::Char('I') if left_normal => app.insert_at_line_start(),
            KeyCode::Char('A') if left_normal => app.insert_at_line_end(),
            KeyCode::Char('o') if left_normal => app.open_below(),
            KeyCode::Char('O') if left_normal => app.open_above(),
            KeyCode::Char('x') if left_normal => app.delete_char_at_cursor(),
            KeyCode::Char('u') if left_normal => app.undo(),
            KeyCode::Char('p') if left_normal => app.paste_yank(1),
            KeyCode::Char('P') if left_normal => app.paste_yank(0),
            KeyCode::Char('d') if left_normal => {
                if app.pending == Some('d') {
                    app.delete_line();
                    app.pending = None;
                } else {
                    app.pending = Some('d');
                }
            }
            KeyCode::Char('y') if left_normal => {
                if app.pending == Some('y') {
                    app.yank_line();
                    app.pending = None;
                } else {
                    app.pending = Some('y');
                }
            }

            KeyCode::Up | KeyCode::Char('k') if left_normal => app.left.move_cursor(CursorMove::Up),
            KeyCode::Down | KeyCode::Char('j') if left_normal => {
                app.left.move_cursor(CursorMove::Down)
            }
            KeyCode::Left | KeyCode::Char('h') if left_normal => {
                app.left.move_cursor(CursorMove::Back)
            }
            KeyCode::Right | KeyCode::Char('l') if left_normal => {
                app.left.move_cursor(CursorMove::Forward)
            }
            KeyCode::Char('G') if left_normal => app.left.move_cursor(CursorMove::Bottom),
            KeyCode::Char('g') if left_normal => {
                if app.pending == Some('g') {
                    app.left.move_cursor(CursorMove::Top);
                    app.pending = None;
                } else {
                    app.pending = Some('g');
                }
            }

            // Same navigation keys, now for the read-only right pane.
            KeyCode::Up | KeyCode::Char('k') => app.right.move_cursor(CursorMove::Up),
            KeyCode::Down | KeyCode::Char('j') => app.right.move_cursor(CursorMove::Down),
            KeyCode::Left | KeyCode::Char('h') => app.right.move_cursor(CursorMove::Back),
            KeyCode::Right | KeyCode::Char('l') => app.right.move_cursor(CursorMove::Forward),
            KeyCode::Char('G') => app.right.move_cursor(CursorMove::Bottom),
            KeyCode::Char('g') => {
                if app.pending == Some('g') {
                    app.right.move_cursor(CursorMove::Top);
                    app.pending = None;
                } else {
                    app.pending = Some('g');
                }
            }

            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                if app.focus == Focus::Left {
                    app.left.input(Event::Key(key));
                } else {
                    app.right.input(Event::Key(key));
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}
