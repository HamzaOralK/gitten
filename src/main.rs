extern crate core;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use std::path::Path;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Terminal};

mod run_app;
use run_app::run_app;
mod utility;

mod git_operations;

mod components;
use components::application::App;


fn main() -> Result<(), io::Error> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "./".to_string());
    if !Path::new(&path).exists() {
        panic!("Path does not exists!");
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::builder().path(path).build();
    let _ = run_app(&mut terminal, app, Duration::from_millis(5000));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
