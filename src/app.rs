#![allow(dead_code)]

use utility::is_repository;
use std::{fmt, fs};
use std::fmt::Debug;
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

#[derive(Debug, Clone, Default)]
pub struct AlfredRepository {
    pub path: String,
    pub folder_name: String,
    pub is_repository: bool,
    pub active_branch_name: String,
}

#[derive(PartialEq)]
pub enum InputMode {
    NORMAL,
    EDITING,
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

pub struct App {
    pub selection: Selection,
    pub repositories: StatefulList<AlfredRepository>,
    pub branches: StatefulList<String>,
    pub tags: StatefulList<String>,
    pub input: String,
    pub input_mode: InputMode
}

impl App {
    pub fn new() -> App {
        let mut content = Vec::new();
        let path = std::env::args().nth(1).unwrap_or("./".to_string());

        App::generate_repository_content(path, &mut content);

        App {
            selection: Selection::REPOSITORIES,
            repositories: StatefulList::with_items(content),
            branches: StatefulList::with_items(vec![]),
            tags: StatefulList::with_items(vec![]),
            input: String::new(),
            input_mode: InputMode::NORMAL
        }
    }

    pub fn change_selection(&mut self, s: Selection) {
        self.selection = s;
    }

    pub fn on_tick(&mut self) {
        ()
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

    pub fn process_input(&mut self) {
        let input: String = self.input.drain(..).collect();

        let commands: Vec<String> = input.split_whitespace().map(|f| {
            f.to_owned()
        }).collect();

        if self.get_selected_repository().is_repository {
            match self.selection {
                Selection::REPOSITORIES => {
                    match commands[0].as_ref() {
                        "co" => { let _ = self.checkout_to_branch(commands[1].to_string()); },
                        "tag" => { let _ = self.create_tag(commands[1].to_string()); },
                        _ => { print!("Unknown command!") }
                    }
                },
                _ => {}
            }
            self.input_mode = InputMode::NORMAL;
        }
    }

    fn checkout_to_branch(&mut self, branch_name: String) {
        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let head = repo.head().unwrap();
            let oid = head.target().unwrap();
            let commit = repo.find_commit(oid).unwrap();
            let branch_name = branch_name.as_str();
            let _branch = repo.branch(
                branch_name,
                &commit,
                false,
            );
            let obj = repo.revparse_single(&("refs/heads/".to_owned() + branch_name)).unwrap();
            let _result = repo.checkout_tree(
                &obj,
                None
            );
            let _result = repo.set_head(&("refs/heads/".to_owned() + branch_name));
            self.get_selected_repository().active_branch_name = branch_name.to_string();
            self.update_repository_details();
        }
    }

    fn create_tag(&mut self, tag_name: String) {
        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let obj = repo.revparse_single(&("refs/heads/".to_owned() + &self.get_selected_repository().active_branch_name)).unwrap();
            let sig = repo.signature().unwrap();
            repo.tag(tag_name.as_str(), &obj, &sig, "", true).unwrap();
            self.update_repository_details();
        }
    }

    fn update_repository_details(&mut self) {
        if self.selection == Selection::REPOSITORIES {
            //Get selected repository
            let rep = get_repository(&self.get_selected_repository().path);
            self.tags = StatefulList::with_items(get_repository_tags(&rep));
            self.branches = StatefulList::with_items(get_repository_branches(&rep));
        }
    }

    fn generate_repository_content(path: String, content: &mut Vec<AlfredRepository>) {
        let paths = fs::read_dir(path).unwrap();

        paths.for_each(|p| {
            let dir = p.unwrap();
            if !dir.file_name().to_str().unwrap().starts_with(".") {
                let repository = get_repository(&dir.path().to_str().unwrap().to_string());
                let active_branch_name = get_repository_active_branch(&repository);
                content.push(
                    AlfredRepository {
                        path: dir.path().to_str().unwrap().to_string(),
                        folder_name: dir.file_name().into_string().unwrap(),
                        is_repository: is_repository(dir.path()),
                        active_branch_name
                    }
                );
            }
        });
        content.sort_by(|a, b| b.folder_name.cmp(&a.folder_name));
    }

    fn get_selected_repository(&mut self) -> &mut AlfredRepository {
        &mut self.repositories.items[self.repositories.state.selected().unwrap()]
    }
}