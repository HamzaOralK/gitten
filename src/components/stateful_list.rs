use std::fmt::Display;
use tui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T: Display> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if !self.items.is_empty() {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if !self.items.is_empty() {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn search(&mut self, input: &str) {
        self.items.iter().enumerate().for_each(|(i, _x)| {
            if self.items[i]
                .to_string()
                .to_lowercase()
                .contains(&input.to_lowercase())
            {
                self.state.select(Some(i));
            }
        });
    }
}