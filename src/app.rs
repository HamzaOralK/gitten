use utility::is_repository;
use std::{fmt, fs};
use std::path::PathBuf;
use tui::widgets::{ListState};
use crate::utility;
use crate::utility::{get_repository, get_repository_active_branch, get_repository_branches, get_repository_tags};

#[derive(PartialEq)]
pub enum Selection {
    REPOSITORIES,
    TAGS,
    BRANCHES
}

impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Selection::REPOSITORIES => "Repositories",
            Selection::TAGS => "Tags",
            Selection::BRANCHES => "Branches"
        })
    }
}

#[derive(Debug, Clone)]
pub struct AlfredRepository {
    pub path: String,
    pub folder_name: String,
    pub is_repository: bool,
    pub active_branch_name: String,
}

pub struct App {
    pub selection: Selection,
    pub selected_repository_path: String,
    pub repositories: StatefulList<AlfredRepository>,
    pub branches: StatefulList<String>,
    pub tags: StatefulList<String>,
    pub tick: u64
}

impl App {
    pub fn new() -> App {
        let mut content = Vec::new();
        let path = std::env::args().nth(1).unwrap_or("./".to_string());

        generate_repository_content(path, &mut content);
        App {
            selection: Selection::REPOSITORIES,
            selected_repository_path: "".to_string(),
            repositories: StatefulList::with_items(content),
            branches: StatefulList::with_items(vec![]),
            tags: StatefulList::with_items(vec![]),
            tick: 0
        }
    }

    pub fn change_selection(&mut self, s: Selection) {
        self.selection = s;
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
    }

    pub fn next(&mut self) {
        match self.selection {
            Selection::REPOSITORIES => {
                self.repositories.next()
            },
            Selection::TAGS => self.tags.next(),
            Selection::BRANCHES => self.branches.next(),
        };
        self.update_repository_details();
    }

    pub fn previous(&mut self) {
        match self.selection {
            Selection::REPOSITORIES => self.repositories.previous(),
            Selection::TAGS => self.tags.previous(),
            Selection::BRANCHES => self.branches.previous(),
        }
        self.update_repository_details();
    }

    fn update_repository_details(&mut self) {
        if self.selection == Selection::REPOSITORIES {
            let temp_value = AlfredRepository{
                path: "".to_string(),
                folder_name: "".to_string(),
                active_branch_name: "".to_string(),
                is_repository: false
            };
            let selected_object = match self.repositories.state.selected() {
                Some(selected) => &self.repositories.items[selected],
                _ => &temp_value
            };
            self.selected_repository_path = selected_object.path.to_string();
            //Get selected repository
            let repository = get_repository(PathBuf::from(&selected_object.path));
            self.tags = StatefulList::with_items(get_repository_tags(&repository));
            self.branches = StatefulList::with_items(get_repository_branches(&repository));
        }
    }
}

fn generate_repository_content(path: String, content: &mut Vec<AlfredRepository>) {
    let paths = fs::read_dir(path).unwrap();

    paths.for_each(|p| {
        let dir = p.unwrap();
        if !dir.file_name().to_str().unwrap().starts_with(".") {
            let repository = get_repository(PathBuf::from(&dir.path()));
            let branch_name = get_repository_active_branch(&repository);
            content.push(
                AlfredRepository {
                    path: dir.path().to_str().unwrap().to_string(),
                    folder_name: dir.file_name().into_string().unwrap(),
                    active_branch_name: branch_name,
                    is_repository: is_repository(dir.path())
                }
            );
        }
    });
    content.sort_by(|a, b| b.folder_name.cmp(&a.folder_name));
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
