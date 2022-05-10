#![allow(dead_code)]

use utility::is_repository;
use std::{fs};
use std::path::PathBuf;
use git2::{Repository};
use tui::widgets::{ListState};
use crate::utility;

pub struct App {
    pub items: StatefulList<(String, String, bool)>,
    pub tick: u64
}

impl App {
    pub fn new() -> App {
        let mut content = Vec::new();
        let paths = fs::read_dir("/Users/hok/tenera").unwrap();

        paths.for_each(|p| {
            let dir = p.unwrap();
            content.push(
                (
                    dir.path().to_str().unwrap().to_string(),
                    dir.file_name().into_string().unwrap(),
                    is_repository(dir.path())
                )
            );
        });

        App {
            items: StatefulList::with_items(content),
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

