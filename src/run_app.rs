use crate::utility::{centered_rect, create_block, create_block_with_selection, create_block_with_title, create_selection_list_from_vector};
use crate::App;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use futures::SinkExt;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{io};
use std::time::{Duration, Instant};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Clear, Paragraph};
use tui::{Frame, Terminal};
use tui::text::{Text};

use crate::git_operations::log::print_log;

use crate::components::{
    logs::Logs,
    modes::InputMode,
    selection::Selection,
};

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    let mut channels = app.channels.0.clone();

    let mut watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            let _ = channels.send(res).await;
        });
    }, Config::default())
    .unwrap();

    watcher
        .watch(app.path.as_ref(), RecursiveMode::Recursive)
        .unwrap();

    loop {
        if let Ok(Some(Ok(event))) = &app.channels.1.try_next() {
            app.update_application_content(event.paths.get(0).unwrap());
        };

        terminal.draw(|f| {
            ui(f, &mut app);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            drop(app.channels.1);
                            return Ok(());
                        },
                        KeyCode::Left => app.repositories.unselect(),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('l') => {
                            if app.repositories.state.selected().is_some() && app.get_selected_repository().is_repository {
                                app.input_mode = InputMode::Logs
                            }
                        },
                        KeyCode::Char('r') => app.change_selection(Selection::Repositories),
                        KeyCode::Char('t') => app.change_selection(Selection::Tags),
                        KeyCode::Char('b') => app.change_selection(Selection::Branches),
                        KeyCode::Char(':') => {
                            if app.repositories.state.selected().is_some() && app.get_selected_repository().is_repository {
                                app.input_mode = InputMode::Editing;
                            } else {
                                app.add_log("Repository Should be selected".to_string())
                            }
                        }
                        KeyCode::Char('/') => {
                            app.input_mode = InputMode::Search;
                        }
                        KeyCode::Char('$') => {
                            if app.repositories.state.selected().is_some() && app.selection == Selection::Repositories {
                                app.input_mode = InputMode::Command;
                            } else {
                                app.add_log("Repository Should be selected".to_string())
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            app.process_input();
                        }
                        KeyCode::Esc => {
                            app.reset_input();
                        }
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                            app.search();
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            app.reset_input();
                        }
                        _ => {}
                    },
                    InputMode::Command => match key.code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            app.run_command_with_path();
                            app.reset_input();
                        }
                        KeyCode::Esc => app.reset_input(),
                        _ => {}
                    },
                    InputMode::Logs => match key.code {
                        KeyCode::Down => { app.scroll_logs_down() },
                        KeyCode::Up => { app.scroll_logs_up(); },
                        KeyCode::Char('q') | KeyCode::Char('l') | KeyCode::Esc => { app.reset_input(); },
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
        .constraints([Constraint::Percentage(97), Constraint::Percentage(3)])
        .split(size);

    // Divides main part into two
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(main_chunks[0]);

    let repository_list = create_selection_list_from_vector(
        &app.repositories.items,
        create_block_with_selection(app, Selection::Repositories),
        Some(&left_chunks[0]),
    );
    f.render_stateful_widget(repository_list, left_chunks[0], &mut app.repositories.state);

    let log_list =
        create_selection_list_from_vector(&app.logs.items, create_block_with_title("Logs"), None);
    f.render_stateful_widget(log_list, left_chunks[1], &mut app.logs.state);

    //Branches and Tags screens
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    // Tags
    let tag_list = create_selection_list_from_vector(
        &app.tags.items,
        create_block_with_selection(app, Selection::Tags),
        None,
    );
    f.render_stateful_widget(tag_list, right_chunks[0], &mut app.tags.state);

    // Branches
    let branch_list = create_selection_list_from_vector(
        &app.branches.items,
        create_block_with_selection(app, Selection::Branches),
        None,
    );
    f.render_stateful_widget(branch_list, right_chunks[1], &mut app.branches.state);

    if app.input_mode == InputMode::Logs {
        let block = Block::default().title("Logs").borders(Borders::ALL);
        let area = centered_rect(90, 90, size);
        f.render_widget(Clear, area); //this clears out the background

        let paragraph = if let Some(logs) = &app.repository_logs {
            Paragraph::new(Text::from(logs.log.clone())).scroll(logs.offset).block(block)
        } else {
            app.repository_logs = Some(Logs{
                log: if let Ok(logs) = print_log(&app.get_selected_repository().path) {
                    logs
                } else {
                    "No logs".to_owned()
                },
                offset: (0, 0)
            });

            if let Some(logs) = &app.repository_logs {
                Paragraph::new(Text::from(logs.log.clone())).scroll(logs.offset)
            } else {
                Paragraph::new("No logs")
            }.block(block)
        };
        f.render_widget(
            paragraph,
            area
        );
    } else {
        app.repository_logs = None;
    }

    // Info at the bottom
    let help = match app.repositories.state.selected() {
        Some(_) => app.generate_help(),
        _ => String::new(),
    };

    let info = match app.input_mode {
        InputMode::Normal => Paragraph::new(help)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .block(create_block())
            .alignment(Alignment::Left),
        InputMode::Editing => {
            Paragraph::new(format!("{} > {}", &app.selection.to_string(), &app.input))
                .style(Style::default().bg(Color::White).fg(Color::Black))
                .block(create_block())
                .alignment(Alignment::Left)
        }
        InputMode::Search => Paragraph::new(format!("Search > {}", &app.input))
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .block(create_block())
            .alignment(Alignment::Left),
        InputMode::Command => Paragraph::new(format!("Command > {}", &app.input))
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .block(create_block())
            .alignment(Alignment::Left),
        InputMode::Logs => Paragraph::new("Logs".to_string())
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .block(create_block())
            .alignment(Alignment::Left),
    };
    f.render_widget(info, chunks[1]);
}
