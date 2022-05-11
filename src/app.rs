use utility::is_repository;
use std::{fs};
use tui::widgets::{ListState};
use crate::utility;

#[derive(Debug, Clone)]
pub struct AlfredRepository {
    pub path: String,
    pub folder_name: String,
    pub is_repository: bool
}

pub struct App {
    pub repositories: StatefulList<AlfredRepository>,
    pub tick: u64
}

impl App {
    pub fn new() -> App {
        let mut content = Vec::new();
        let path = std::env::args().nth(1).unwrap_or("./".to_string());

        generate_repository_content(path, &mut content);
        App {
            repositories: StatefulList::with_items(content),
            tick: 0
        }
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
    }
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T: Clone> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    #[allow(dead_code)]
    fn add(&mut self, items: Vec<T>) {
        self.items.push(items[0].clone())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

fn generate_repository_content(path: String, content: &mut Vec<AlfredRepository>) {
    let paths = fs::read_dir(path).unwrap();

    paths.for_each(|p| {
        let dir = p.unwrap();
        if !dir.file_name().to_str().unwrap().starts_with(".") {
            content.push(
                AlfredRepository {
                    path: dir.path().to_str().unwrap().to_string(),
                    folder_name: dir.file_name().into_string().unwrap(),
                    is_repository: is_repository(dir.path())
                }
            );
        }
    });
    content.sort_by(|a, b| b.folder_name.cmp(&a.folder_name));
}