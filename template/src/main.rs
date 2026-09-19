//! Точка входа: event loop (ввод + тик метрик + отрисовка).

mod model;
mod theme;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use model::{App, Focus, Tab};
use theme::{Palette, Theme};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme::new(Palette::DARK);
    let mut app = App::new();
    let mut prev_g = false;

    let result = run(&mut terminal, &theme, &mut app, &mut prev_g);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    theme: &Theme,
    app: &mut App,
    prev_g: &mut bool,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app, theme))?;

        if event::poll(Duration::from_millis(500))? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key(app, key, prev_g) {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        } else {
            app.tick();
        }
    }
}

/// Возвращает `true`, когда нужно выйти из приложения.
fn handle_key(app: &mut App, key: KeyEvent, prev_g: &mut bool) -> bool {
    if app.show_help {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.show_help = false;
                false
            }
            _ => false,
        };
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        KeyCode::Char('?') => {
            app.show_help = true;
            *prev_g = false;
            false
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.next();
            *prev_g = false;
            false
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.prev();
            *prev_g = false;
            false
        }
        KeyCode::Char('g') => {
            if *prev_g {
                app.first();
                *prev_g = false;
            } else {
                *prev_g = true;
            }
            false
        }
        KeyCode::Char('G') => {
            app.last();
            *prev_g = false;
            false
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.focus = Focus::List;
            *prev_g = false;
            false
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.focus = Focus::Detail;
            *prev_g = false;
            false
        }
        KeyCode::Tab => {
            app.tab = match app.tab {
                Tab::Overview => Tab::Logs,
                Tab::Logs => Tab::Overview,
            };
            *prev_g = false;
            false
        }
        KeyCode::Enter => {
            app.toggle();
            *prev_g = false;
            false
        }
        KeyCode::Char('r') => {
            app.restart();
            *prev_g = false;
            false
        }
        KeyCode::Char('s') => {
            app.stop();
            *prev_g = false;
            false
        }
        KeyCode::Char('S') => {
            app.start();
            *prev_g = false;
            false
        }
        _ => {
            *prev_g = false;
            false
        }
    }
}
