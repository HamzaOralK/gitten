use std::{fmt, fs};
use std::fmt::{Debug, Display, Formatter};
use std::path::{PathBuf};
use std::process::Command;
use git2::{PushOptions, ResetType};
use std::string::String;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{ListItem, ListState};
use crate::pull::{fetch_branches_repository_from_remote, fetch_repository_from_remote};
use crate::repo::{get_files_changed, get_repository, get_repository_active_branch, get_repository_branches, get_repository_tags, git_credentials_callback, is_repository};

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

impl AlfredRepositoryItem {
    fn set_active_branch_name(&mut self, name: String) {
        self.active_branch_name = name;
    }

    fn set_files_changed(&mut self, files_changed: usize) {
        self.files_changed = files_changed;
    }
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
    Search,
    Command
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T: Clone + Display> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
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
            None => 0
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn search(&mut self, input: &str) {
        self.items.iter().enumerate().for_each(|(i, _x)| {
            if self.items[i].to_string().to_lowercase().contains(&input.to_lowercase()) {
                self.state.select(Some(i));
            }
        });
    }
}

pub struct App {
    pub selection: Selection,
    pub repositories: StatefulList<AlfredRepositoryItem>,
    pub branches: StatefulList<AlfredStringItems>,
    pub tags: StatefulList<AlfredStringItems>,
    pub input: String,
    pub input_mode: InputMode,
    pub logs: StatefulList<String>,
    pub path: String
}

impl App {
    pub fn new() -> App {
        let mut content = Vec::new();
        let path = std::env::args().nth(1).unwrap_or_else(|| "./".to_string());

        App::generate_application_content(&path, &mut content);

        App {
            selection: Selection::Repositories,
            repositories: StatefulList::with_items(content),
            branches: StatefulList::with_items(vec![]),
            tags: StatefulList::with_items(vec![]),
            input: String::new(),
            input_mode: InputMode::Normal,
            logs: StatefulList::with_items(vec![]),
            path
        }
    }

    pub fn change_selection(&mut self, s: Selection) {
        self.selection = s;
    }

    pub fn on_tick(&mut self) {
        self.update_application_content();
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
                        "co" => { let _ = self.checkout_to_branch(commands.get(1) ); },
                        "tag" => { let _ = self.create_tag(commands.get(1)); },
                        "rh" => { let _ = self.reset_selected_repository(ResetType::Hard); },
                        "pull" => { let _ = self.pull_remote(commands.get(1)); }
                        "fetch" => { let _ = self.fetch_remote(commands.get(1)); }
                        _ => { self.add_log("Unknown command!".to_string()) }
                    }
                },
                Selection::Branches => {
                    match commands[0].as_ref() {
                        "push" => { let _ = self.push_remote(commands.get(1), true); },
                        _ => { self.add_log("Unknown command!".to_string()) }
                    }
                },
                Selection::Tags => {
                    match commands[0].as_ref() {
                        "push" => { let _ = self.push_remote(commands.get(1), false); },
                        _ => { self.add_log("Unknown command!".to_string()) }
                    }
                }
            }
            self.input_mode = InputMode::Normal;
        }
    }

    fn push_remote(&mut self, remote: Option<&String>, is_branch: bool) {
        let remote = match remote {
            Some(b) => b,
            None => { self.add_log("remote name must not be null".to_string()); return }
        };

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(git_credentials_callback);

        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let mut opts = PushOptions::new();
            opts.remote_callbacks(callbacks);
            let mut remote = repo.find_remote(remote.as_str()).unwrap();

            let ref_spec = if is_branch {
                if let Some(b) = self.branches.state.selected() {
                    format!("refs/heads/{}", &self.branches.items[b].to_string())
                } else {
                    self.add_log("Please select a branch!".to_string());
                    return
                }
            } else if let Some(t) = self.tags.state.selected() {
                    format!("refs/tags/{}", &self.tags.items[t].to_string())
            } else {
                self.add_log("Please select a tag!".to_string());
                return
            };

            match remote.push(&[&ref_spec], Some(&mut opts)) {
                Ok(()) => self.add_log("Push is successful!".to_string()),
                Err(e) => { self.add_log("Error: ".to_owned() + e.message()) }
            };
        }
    }

    fn pull_remote(&mut self, remote: Option<&String>) {

        let remote = match remote {
            Some(b) => b,
            None => { self.add_log("remote name must not be null".to_string()); return }
        };

        let selected_repository = self.get_selected_repository();
        let branch_name = selected_repository.active_branch_name.as_str().to_owned();
        let repository = get_repository(&selected_repository.path).unwrap();

        match fetch_repository_from_remote(remote.as_str(), branch_name.as_str(), &repository) {
            Ok(r) => {
                self.add_log(r);
            },
            Err(e) => {
                self.add_log(format!("Error: {}", e.message()))
            }
        };
    }

    fn fetch_remote(&mut self, remote: Option<&String>) {

        let remote = match remote {
            Some(b) => b,
            None => { self.add_log("remote name must not be null".to_string()); return }
        };

        let selected_repository = self.get_selected_repository();
        let repository = get_repository(&selected_repository.path).unwrap();

        match fetch_branches_repository_from_remote( remote.as_str(), &repository) {
            Ok(message) => {
                self.add_log(message);
                self.update_repository_details();
            },
            Err(e) => self.add_log(e.message().to_string())
        }
    }

    fn checkout_to_branch(&mut self, branch_name: Option<&String>) {

        let branch_name = match branch_name {
            Some(b) => b,
            None => { self.add_log("Branch name must not be null".to_string()); return }
        };

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
                    self.add_log("Checkout is successful!".to_string());
                },
                Err(e) => {
                    self.add_log(format!("Error: {}", e.message()))
                }
            };
        }
    }

    fn create_tag(&mut self, tag_name: Option<&String>) {

        let tag_name = match tag_name {
            Some(b) => b,
            None => { self.add_log("Tag name must not be null".to_string()); return }
        };

        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let obj = repo.revparse_single(&("refs/heads/".to_owned() + &self.get_selected_repository().active_branch_name)).unwrap();
            let sig = repo.signature().unwrap();
            match repo.tag(tag_name.as_str(), &obj, &sig, format!("Release {}", tag_name).as_str(), true) {
                Ok(_oid) => { self.add_log(String::from("Tag creation is successful!")) },
                Err(e) => { self.add_log(format!("Error: {}", e.message()))  }
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

    fn generate_application_content(path: &String, content: &mut Vec<AlfredRepositoryItem>) {
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
        content.sort_by(|a, b| {
            a.folder_name.to_lowercase().cmp(&b.folder_name.to_lowercase())
        });
    }

    fn update_application_content(&mut self) {
        self.repositories.items.iter_mut().for_each(|f| {
            let repository = get_repository(&f.path);
            let mut is_repository = false;
            let mut active_branch_name = String::new();

            if repository.is_some() {
                is_repository = true;
                active_branch_name = get_repository_active_branch(&repository);
            }

            let files_changed = get_files_changed(&repository).unwrap_or(0);
            if f.is_repository != is_repository || f.active_branch_name != active_branch_name {
                f.set_active_branch_name(active_branch_name);
                f.set_files_changed(files_changed);
            };
        });
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
                    self.get_selected_repository().files_changed = 0;
                    self.add_log(String::from("Reset Successful!"))
                },
                Err(_) => self.add_log(String::from("Could not reset!"))
            };
        }
    }

    fn add_log(&mut self, message: String) {
        if self.repositories.state.selected().is_some() {
            self.logs.items.push(format!("{} - {}", self.get_repository_info(), message));
        } else {
            self.logs.items.push(message);
        }

        self.logs.state.select(Some(self.logs.items.len()));
    }

    fn get_repository_info(&self) -> String {
        let r = &self.repositories.items[self.repositories.state.selected().unwrap()];
        format!("{} - {}", r.folder_name, r.active_branch_name)
    }

    pub fn generate_help(&self) -> String {
        match self.selection {
            Selection::Repositories => String::from(":co | :tag | :rh | :pull <remote> | :fetch <remote> | q"),
            Selection::Branches => String::from(":push <remote> | q"),
            Selection::Tags => String::from(":push <remote> | q"),
        }
    }

    pub fn search(&mut self) {
        let input = self.input.as_str();
        match self.selection {
            Selection::Tags => self.tags.search(input),
            Selection::Branches => self.branches.search(input),
            Selection::Repositories => self.repositories.search(input),
        }
    }

    pub fn reset_input(&mut self) {
        self.input = String::new();
        self.input_mode = InputMode::Normal;
    }

    pub fn run_command_with_path(&mut self) {
        if let Err(e) = Command::new(&self.input).arg(&self.get_selected_repository().path).output() {
            self.add_log(e.to_string())
        }
    }
}
