use utility::is_repository;
use std::{fmt, fs};
use std::fmt::Debug;
use git2::{Cred, CredentialType, PushOptions};
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
                if self.items.len() > 0 {
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
                if self.items.len() > 0 {
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
    pub repositories: StatefulList<AlfredRepository>,
    pub branches: StatefulList<String>,
    pub tags: StatefulList<String>,
    pub input: String,
    pub input_mode: InputMode,
    pub message: Option<String>,
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
            input_mode: InputMode::NORMAL,
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
                Selection::BRANCHES => {
                    match commands[0].as_ref() {
                        "p" => { let _ = self.push_origin(commands[1].to_string(), true); },
                        _ => { print!("Unknown command!") }
                    }
                },
                Selection::TAGS => {
                    match commands[0].as_ref() {
                        "p" => { let _ = self.push_origin(commands[1].to_string(), false); },
                        _ => { print!("Unknown command!") }
                    }
                }
            }
            self.input_mode = InputMode::NORMAL;
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
            match repo.tag_lightweight(tag_name.as_str(), &obj, true) {
                Ok(_oid) => { self.set_message(Some(String::from("Tag creation is successful!"))) },
                Err(e) => { self.set_message(Some(format!("Error: {}", e.message())))  }
            };
            self.update_repository_details();
        }
    }

    fn update_repository_details(&mut self) {
        if self.selection == Selection::REPOSITORIES {
            //Get selected repository
            let rep = get_repository(&self.get_selected_repository().path);
            self.tags.unselect();
            self.tags = StatefulList::with_items(get_repository_tags(&rep));
            self.branches.unselect();
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

    fn set_message(&mut self, message: Option<String>) {
        self.message = message
    }
}

fn git_credentials_callback(
    _url: &str,
    user_from_url: Option<&str>,
    cred_types_allowed: CredentialType,
) -> Result<Cred, git2::Error> {
    let user = user_from_url.unwrap();

    if cred_types_allowed.contains(CredentialType::SSH_KEY) {
        let private_key = dirs::home_dir().unwrap().join(".ssh").join("id_rsa");
        let cred = Cred::ssh_key(user, None, &private_key, None);
        return cred;
    }

    return Err(git2::Error::from_str("no credential option available"));
}