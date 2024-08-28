use std::fs;
use std::io;
use std::path::Path;
use std::env;

use chrono::NaiveDate;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    },
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

mod backend;

struct App {
    dates: Vec<NaiveDate>,
    selected_index: usize,
    start_index: usize,
    base_path: String,
}

fn main() -> Result<(), io::Error> {
    let config_home = format!("{}/tuidiary",
        env::var("XDG_CONFIG_HOME").unwrap_or(String::from("~/.config")));
    if !Path::new(&config_home).exists() {
        let _ = fs::create_dir_all(config_home);
    }

    let diary_dir = env::var("DIARY_DIR")
        .expect("DIARY_DIR environment variable");

    let config_editor = env::var("EDITOR").unwrap_or(String::from("vim"));

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App {
        dates: backend::get_entries_in_path(diary_dir.clone()),
        selected_index: 0,
        start_index: 0,
        base_path: diary_dir,
    };

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if app.selected_index > 0 {
                        app.selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.selected_index < app.dates.len() - 1 {
                        app.selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    let date = app.dates[app.selected_index];
                    backend::edit_entry(&config_editor, &app.base_path, &date)?;
                    terminal.clear()?;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui<B: tui::backend::Backend>(f: &mut tui::Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()
        ).split(f.size());

    let left_panel_height = chunks[0].height as usize;
    let visible_items = left_panel_height - 2;

    if app.selected_index > visible_items + app.start_index - 1 {
        app.start_index += 1;
    } else if app.selected_index < app.start_index {
        app.start_index -= 1;
    }

    let dates: Vec<ListItem> = app
        .dates
        .iter()
        .skip(app.start_index)
        .take(visible_items)
        .enumerate()
        .map(|(i, &date)| {
            ListItem::new(Spans::from(vec![Span::styled(
                if i == app.selected_index - app.start_index {
                    date.format("=> %Y-%m-%d").to_string()
                } else {
                    date.format("%Y-%m-%d").to_string()
                },
                if backend::get_entry_path(&app.base_path, &date).exists {
                    Style::default()
                } else {
                    Style::default().add_modifier(Modifier::BOLD)
                },
            )]))
        })
        .collect();

    let dates_list = List::new(dates)
        .block(Block::default().title("Dates").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(dates_list, chunks[0]);

    let content = if let Some(date) = app.dates.get(app.selected_index) {
        backend::read_entry(&app.base_path, date)
    } else {
        String::new()
    };

    let preview = Paragraph::new(content).block(
        Block::default().title(
            app.dates[app.selected_index].format("%A %Y-%m-%d").to_string()
        ).borders(Borders::ALL)
    );

    f.render_widget(preview, chunks[1]);
}
