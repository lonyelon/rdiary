use std::io;
use std::path::Path;
use std::env;
use std::process::Command;

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
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

mod backend;

struct App {
    dates: Vec<backend::DiaryEntry>,
    selected_index: usize,
    start_index: usize,
    path: String,
    template_path: String,
    editor: String,
}

fn main() -> Result<(), io::Error> {
    let diary_dir = env::var("RDIARY_DIARY_DIR")
        .expect("RDIARY_DIARY_DIR environment variable");
    let mut app = App {
        dates: backend::get_entries_in_path(&diary_dir),
        selected_index: 0,
        start_index: 0,
        path: diary_dir,
        template_path: env::var("RDIARY_TEMPLATE_PATH").unwrap_or(
            String::new()
        ),
        editor: env::var("EDITOR").unwrap_or(String::from("vim")),
    };

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('g') => {
                    app.selected_index = app.dates.len() - 1;
                }
                KeyCode::Char('G') => {
                    app.selected_index = 0;
                    app.start_index = 0;
                }
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
                    edit_entry(&app, &app.dates[app.selected_index])?;
                    terminal.clear()?;

                    // TODO Maybe this refresh should only be done when the
                    //      entry did not exist previously.
                    app.dates.clear();
                    app.dates = backend::get_entries_in_path(&app.path);
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
            [Constraint::Percentage(15), Constraint::Percentage(80)].as_ref()
        ).split(f.size());

    let left_panel_height = chunks[0].height as usize;
    let visible_items = left_panel_height - 2;

    let lowest_viewable_item = visible_items + app.start_index - 1;
    if app.selected_index > lowest_viewable_item {
        app.start_index += app.selected_index - lowest_viewable_item;
    } else if app.selected_index < app.start_index {
        app.start_index -= 1;
    }

    let dates: Vec<ListItem> = app
        .dates
        .iter()
        .skip(app.start_index)
        .take(visible_items)
        .enumerate()
        .map(|(i, &ref date)| {
            let mut style = Style::default();
            if !date.exists {
                style = style.fg(Color::Red).add_modifier(Modifier::ITALIC);
            }
            if i == app.selected_index - app.start_index {
                style = style.add_modifier(Modifier::BOLD);
            }
            ListItem::new(Spans::from(vec![Span::styled(
                if i == app.selected_index - app.start_index {
                    date.date.format("%Y-%m-%d <=").to_string()
                } else {
                    date.date.format("%Y-%m-%d").to_string()
                },
                style,
            )]))
        })
        .collect();

    let dates_list = List::new(dates)
        .block(Block::default().title("Dates").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(dates_list, chunks[0]);

    let content = if let Some(entry) = app.dates.get(app.selected_index) {
        backend::read_entry(&entry)
    } else {
        String::new()
    };

    let preview = Paragraph::new(content).block(
        Block::default().title(
            app.dates[app.selected_index].date.format("%A %Y-%m-%d").to_string()
        ).borders(Borders::ALL)
    );

    f.render_widget(preview, chunks[1]);
}

fn edit_entry(app: &App, entry: &backend::DiaryEntry) -> io::Result<()> {
    if !Path::new(&entry.path).exists() {
        if app.template_path.is_empty() {
            std::fs::File::create(&entry.path)?;
        } else {
            std::fs::copy(&app.template_path, &entry.path)?;
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let status = Command::new(&app.editor).arg(&entry.path).status()?;

    if !status.success() {
        eprintln!("Failed to open editor");
    }
    
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    Ok(())
}
