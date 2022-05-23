use utility::is_repository;
use std::{fmt, fs};
use std::fmt::{Debug, Display, Formatter};
use git2::{PushOptions, ResetType};
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{ListItem, ListState};

use crate::utility;
use crate::utility::{fetch_repository_from_remote, get_files_changed, get_repository, get_repository_active_branch, get_repository_branches, get_repository_tags, git_credentials_callback};

pub trait ConvertableToListItem {
    fn convert_to_list_item(&self, chunk: Option<&Rect>) -> ListItem;
}

#[derive(PartialEq)]
pub enum Selection {
    Repositories,
    Tags,
    Branches
}

impl Display for Selection {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Selection::Repositories => "(R)epositories",
            Selection::Tags => "(T)ags",
            Selection::Branches => "(B)ranches"
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct AlfredRepositoryItem {
    pub path: String,
    pub folder_name: String,
    pub is_repository: bool,
    pub active_branch_name: String,
    pub files_changed: usize,
}

impl Display for AlfredRepositoryItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.folder_name)
    }
}

impl ConvertableToListItem for AlfredRepositoryItem {
    fn convert_to_list_item(&self, chunk: Option<&Rect>) -> ListItem {
        let mut lines: Spans = Spans::default();
        let mut line_color = Color::Reset;
        if self.is_repository {
            let mut margin = 4;
            if self.files_changed > 0 {
                margin += 1;
            }
            let repeat_time = if chunk.unwrap().width > ((self.active_branch_name.len() as u16) + (self.folder_name.len() as u16) + margin) {
                chunk.unwrap().width - ((self.active_branch_name.len() as u16) + (self.folder_name.len() as u16) + margin)
            } else {
                0
            };
            lines.0.push(Span::from(self.folder_name.clone()));
            lines.0.push(Span::from(" ".repeat((repeat_time) as usize)));
            lines.0.push(Span::raw("("));
            lines.0.push(Span::from(self.active_branch_name.to_string()));
            lines.0.push(Span::from(if self.files_changed > 0 { "*" } else { "" }));
            lines.0.push(Span::raw(")"));
            line_color = Color::Green
        } else {
            lines.0.push(Span::from(self.folder_name.clone()));
        }
        ListItem::new(lines).style(Style::default().fg(Color::White).bg(line_color))
    }
}

type AlfredStringItems = String;

impl ConvertableToListItem for AlfredStringItems {
    fn convert_to_list_item(&self, _chunk: Option<&Rect>) -> ListItem {
        ListItem::new(vec![
            Spans::from(vec![
                Span::raw(self.to_string())
            ])
        ])
    }
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
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
            None => 0
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub struct App {
    pub selection: Selection,
    pub repositories: StatefulList<AlfredRepositoryItem>,
    pub branches: StatefulList<AlfredStringItems>,
    pub tags: StatefulList<AlfredStringItems>,
    pub input: String,
    pub input_mode: InputMode,
    pub message: Option<String>,
}

impl App {
    pub fn new() -> App {
        let mut content = Vec::new();
        let path = std::env::args().nth(1).unwrap_or_else(|| "./".to_string());

        App::generate_repository_content(path, &mut content);

        App {
            selection: Selection::Repositories,
            repositories: StatefulList::with_items(content),
            branches: StatefulList::with_items(vec![]),
            tags: StatefulList::with_items(vec![]),
            input: String::new(),
            input_mode: InputMode::Normal,
            message: None
        }
    }

    pub fn change_selection(&mut self, s: Selection) {
        self.selection = s;
    }

    pub fn on_tick(&mut self) {
        self.message = None;
    }

    pub fn next(&mut self) {
        match self.selection {
            Selection::Repositories => {
                self.repositories.next()
            },
            Selection::Tags => self.tags.next(),
            Selection::Branches => self.branches.next(),
        };
        self.update_repository_details();
    }

    pub fn previous(&mut self) {
        match self.selection {
            Selection::Repositories => self.repositories.previous(),
            Selection::Tags => self.tags.previous(),
            Selection::Branches => self.branches.previous(),
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
                Selection::Repositories => {
                    match commands[0].as_ref() {
                        "co" => { let _ = self.checkout_to_branch(commands[1].to_string()); },
                        "tag" => { let _ = self.create_tag(commands[1].to_string()); },
                        "rh" => { let _ = self.reset_selected_repository(ResetType::Hard); },
                        "pull" => { let _ = self.pull_origin(commands[1].to_string()); }
                        _ => { print!("Unknown command!") }
                    }
                },
                Selection::Branches => {
                    match commands[0].as_ref() {
                        "push" => { let _ = self.push_origin(commands[1].to_string(), true); },
                        _ => { print!("Unknown command!") }
                    }
                },
                Selection::Tags => {
                    match commands[0].as_ref() {
                        "push" => { let _ = self.push_origin(commands[1].to_string(), false); },
                        _ => { print!("Unknown command!") }
                    }
                }
            }
            self.input_mode = InputMode::Normal;
        }
    }

    pub fn generate_help(&self) -> String {
        match self.selection {
            Selection::Repositories => String::from(":co | :tag | :rh | :pull <remote> | q"),
            Selection::Branches => String::from(":push <remote> | q"),
            Selection::Tags => String::from(":push <remote> | q"),
        }
    }

    fn push_origin(&mut self, origin: String, is_branch: bool) {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(git_credentials_callback);

        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let mut opts = PushOptions::new();
            opts.remote_callbacks(callbacks);
            let mut remote = repo.find_remote(origin.as_str()).unwrap();

            let ref_spec = if is_branch {
                format!("refs/heads/{}", &self.branches.items[self.branches.state.selected().unwrap()].to_string())
            } else {
                format!("refs/tags/{}", &self.tags.items[self.tags.state.selected().unwrap()].to_string())
            };

            match remote.push(&[&ref_spec], Some(&mut opts)) {
                Ok(()) => self.set_message(Some(String::from("Push is successful!"))),
                Err(e) => { self.set_message(Some(format!("Error: {}", e.message()))) }
            };
        }
    }

    fn pull_origin(&mut self, origin: String) {
        let selected_repository = self.get_selected_repository();
        let branch_name = selected_repository.active_branch_name.as_str().to_owned();
        let repository = get_repository(&selected_repository.path).unwrap();

        self.set_message(Some(fetch_repository_from_remote(origin.as_str(), branch_name.as_str(), &repository).unwrap()));
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
            match repo.checkout_tree(&obj, None) {
                Ok(()) => {
                    let _result = repo.set_head(&("refs/heads/".to_owned() + branch_name));
                    self.get_selected_repository().active_branch_name = branch_name.to_string();
                    self.update_repository_details();
                    self.set_message(Some(String::from("Checkout is successful!")));
                },
                Err(e) => {
                    self.set_message(Some(format!("Error: {}", e.message())))
                }
            };
        }
    }

    fn create_tag(&mut self, tag_name: String) {
        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let obj = repo.revparse_single(&("refs/heads/".to_owned() + &self.get_selected_repository().active_branch_name)).unwrap();
            let sig = repo.signature().unwrap();
            match repo.tag(tag_name.as_str(), &obj, &sig, format!("Release {}", tag_name).as_str(), true) {
                Ok(_oid) => { self.set_message(Some(String::from("Tag creation is successful!"))) },
                Err(e) => { self.set_message(Some(format!("Error: {}", e.message())))  }
            };
            self.update_repository_details();
        }
    }

    fn update_repository_details(&mut self) {
        if self.selection == Selection::Repositories {
            //Get selected repository
            let rep = get_repository(&self.get_selected_repository().path);
            self.tags.unselect();
            self.tags = StatefulList::with_items(get_repository_tags(&rep));
            self.branches.unselect();
            self.branches = StatefulList::with_items(get_repository_branches(&rep));
        }
    }

    fn generate_repository_content(path: String, content: &mut Vec<AlfredRepositoryItem>) {
        let paths = fs::read_dir(path).unwrap();

        paths.for_each(|p| {
            let dir = p.unwrap();
            if !dir.file_name().to_str().unwrap().starts_with('.') {
                let repository = get_repository(&dir.path().to_str().unwrap().to_string());
                let active_branch_name = get_repository_active_branch(&repository);
                let files_changed = get_files_changed(&repository).unwrap_or(0);
                content.push(
                    AlfredRepositoryItem {
                        path: dir.path().to_str().unwrap().to_string(),
                        folder_name: dir.file_name().into_string().unwrap(),
                        is_repository: is_repository(dir.path()),
                        active_branch_name,
                        files_changed
                    }
                );
            }
        });
        content.sort_by(|a, b| b.folder_name.cmp(&a.folder_name));
    }

    fn get_selected_repository(&mut self) -> &mut AlfredRepositoryItem {
        &mut self.repositories.items[self.repositories.state.selected().unwrap()]
    }

    fn reset_selected_repository(&mut self, reset_type: ResetType) {
        if let Some(r) = get_repository(&self.get_selected_repository().path) {
            let head = r.head().unwrap();
            let obj = r.find_object(head.target().unwrap(), None).unwrap();
            match r.reset(&obj, reset_type, None) {
                Ok(()) => {
                    self.get_selected_repository().files_changed = 0
                },
                Err(_) => self.set_message(Some(String::from("Could not reset!")))
            };
        }
    }

    fn set_message(&mut self, message: Option<String>) {
        self.message = message
    }
}
