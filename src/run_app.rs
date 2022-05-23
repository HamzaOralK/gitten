use std::{io};
use std::time::{Duration, Instant};
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use tui::backend::Backend;
use tui::{Frame, Terminal};
use tui::widgets::{Paragraph};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use crate::{App};
use crate::app::{InputMode, Selection};
use crate::utility::{create_block, create_block_with_title, create_selection_list_from_vector};

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
                match app.input_mode {
                    InputMode::Normal=> match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Left => app.repositories.unselect(),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('r') => app.change_selection(Selection::Repositories),
                        KeyCode::Char('t') => app.change_selection(Selection::Tags),
                        KeyCode::Char('b') => app.change_selection(Selection::Branches),
                        KeyCode::Char(':') => {
                            if app.repositories.state.selected().is_some() {
                                app.input_mode = InputMode::Editing;
                            };
                        },
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        },
                        KeyCode::Backspace => {
                            app.input.pop();
                        },
                        KeyCode::Enter => {
                            app.process_input();
                        },
                        KeyCode::Esc => {
                            app.input = String::new();
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
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
                Constraint::Percentage(97),
                Constraint::Percentage(3)
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

    let repository_list = create_selection_list_from_vector(&app.repositories.items, create_block_with_title(app, Selection::Repositories), Some(&main_chunks[0]));
    f.render_stateful_widget(repository_list, main_chunks[0], &mut app.repositories.state);

    //Branches and Tags screens
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ]
        )
        .split(main_chunks[1]);

    // Tags
    let tag_list = create_selection_list_from_vector(&app.tags.items, create_block_with_title(app, Selection::Tags), None);
    f.render_stateful_widget(tag_list, right_chunks[0], &mut app.tags.state);

    // Branches
    let branch_list = create_selection_list_from_vector(&app.branches.items, create_block_with_title(app, Selection::Branches), None);
    f.render_stateful_widget(branch_list, right_chunks[1], &mut app.branches.state);

    let message = match &app.message {
        Some(message) => {
            message.to_string()
        },
        None => {
            match app.repositories.state.selected() {
                Some(_) => app.generate_help(),
                _ => String::new()
            }
        }
    };

    // Info at the bottom
    let paragraph = match app.input_mode {
        InputMode::Normal => Paragraph::new(message)
                .style(Style::default().bg(Color::White).fg(Color::Black))
                .block(create_block())
                .alignment(Alignment::Left),
        InputMode::Editing => Paragraph::new(format!("{} > {}", &app.selection.to_string(), &app.input))
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .block(create_block())
            .alignment(Alignment::Left)
    };
    f.render_widget(paragraph, chunks[1]);
}
