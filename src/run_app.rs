use std::{io};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::string::String;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use tui::backend::Backend;
use tui::{Frame, Terminal};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use crate::{App};
use crate::utility::{get_repository, get_repository_tags};

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| {
            ui( f, &mut app);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now()
        }
    }
}

fn ui<'a, B: Backend>(f: &'a mut Frame<B>, app: &'a mut App) {
    let size = f.size();

    // Big chunk divides screen for part and bottom info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(95),
                Constraint::Percentage(5)
            ]
        )
        .split(size);

    // Divides main part into two
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ]
        )
        .split(chunks[0]);

    let create_block = || {
        let b = Block::default();
        b.borders(Borders::NONE)
            .style(Style::default().bg(Color::Black).fg(Color::White))
    };

    // Files & folders
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(i.1.clone())];
            if i.2 {
                ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Green))
            } else {
                ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
            }

        })
        .collect();

    let items = List::new(items)
        .block(create_block().title("List"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    f.render_stateful_widget(items, main_chunks[0], &mut app.items.state);

    // Other half
    let temp_value = &("".to_string(), "".to_string(), false);
    let selected_object = match app.items.state.selected() {
        Some(selected) => &app.items.items[selected],
        _ => temp_value
    };

    let repository = get_repository(PathBuf::from(&selected_object.0));
    let paragraph = Paragraph::new(format!("{}", get_repository_tags(repository)))
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(create_block().title("Tags"))
        .alignment(Alignment::Left);
    f.render_widget(paragraph, main_chunks[1]);

    // Info at the bottom
    let paragraph = Paragraph::new(format!("{}",  selected_object.0))
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(create_block())
        .alignment(Alignment::Left);

    f.render_widget(paragraph, chunks[1]);
}