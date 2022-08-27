use crate::git_operations::pull::{fetch_branches_repository_from_remote, fetch_repository_from_remote};
use crate::git_operations::repo::{
    get_files_changed, get_repository, get_repository_active_branch, get_repository_branches,
    get_repository_tags, git_credentials_callback, is_repository,
};
use futures::channel::mpsc::{channel, Receiver, Sender};
use git2::{PushOptions, ResetType};
use notify::Event;
use std::path::{Path};
use std::process::Command;
use std::string::String;
use std::fs;
use crate::components::{
    items::{GittenRepositoryItem, GittenStringItem},
    logs::Logs,
    modes::InputMode,
    selection::Selection,
    stateful_list::StatefulList,
};

pub struct App {
    pub selection: Selection,
    pub repositories: StatefulList<GittenRepositoryItem>,
    pub branches: StatefulList<GittenStringItem>,
    pub tags: StatefulList<GittenStringItem>,
    pub input: String,
    pub input_mode: InputMode,
    pub logs: StatefulList<String>,
    pub path: String,
    pub repository_logs: Option<Logs>,
    pub channels: (
        Sender<notify::Result<Event>>,
        Receiver<notify::Result<Event>>,
    ),
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn change_selection(&mut self, s: Selection) {
        self.selection = s;
    }

    pub fn on_tick(&mut self) {}

    pub fn next(&mut self) {
        match self.selection {
            Selection::Repositories => self.repositories.next(),
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

        let commands: Vec<String> = input.split_whitespace().map(|f| f.to_owned()).collect();

        if self.get_selected_repository().is_repository && !commands.is_empty() {
            match self.selection {
                Selection::Repositories => match commands[0].as_ref() {
                    "co" => {
                        self.checkout_to_branch(commands.get(1));
                    }
                    "tag" => {
                        self.create_tag(commands.get(1));
                    }
                    "rh" => {
                        self.reset_selected_repository(ResetType::Hard);
                    }
                    "pull" => {
                        self.pull_remote(commands.get(1));
                    }
                    "fetch" => {
                        self.fetch_remote(commands.get(1));
                    }
                    _ => self.add_log("Unknown command!".to_string()),
                },
                Selection::Branches => match commands[0].as_ref() {
                    "push" => {
                        self.push_remote(commands.get(1), true);
                    }
                    _ => self.add_log("Unknown command!".to_string()),
                },
                Selection::Tags => match commands[0].as_ref() {
                    "push" => {
                        self.push_remote(commands.get(1), false);
                    }
                    _ => self.add_log("Unknown command!".to_string()),
                },
            }
        }
        self.input_mode = InputMode::Normal;
    }

    fn push_remote(&mut self, remote: Option<&String>, is_branch: bool) {
        let remote = match remote {
            Some(b) => b,
            None => {
                self.add_log("remote name must not be null".to_string());
                return;
            }
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
                    return;
                }
            } else if let Some(t) = self.tags.state.selected() {
                format!("refs/tags/{}", &self.tags.items[t].to_string())
            } else {
                self.add_log("Please select a tag!".to_string());
                return;
            };

            match remote.push(&[&ref_spec], Some(&mut opts)) {
                Ok(()) => self.add_log("Push is successful!".to_string()),
                Err(e) => self.add_log("Error: ".to_owned() + e.message()),
            };
        }
    }

    fn pull_remote(&mut self, remote: Option<&String>) {
        let remote = match remote {
            Some(b) => b,
            None => {
                self.add_log("remote name must not be null".to_string());
                return;
            }
        };

        let selected_repository = self.get_selected_repository();
        let branch_name = selected_repository.active_branch_name.as_str().to_owned();
        let repository = get_repository(&selected_repository.path).unwrap();

        match fetch_repository_from_remote(remote.as_str(), branch_name.as_str(), &repository) {
            Ok(r) => {
                self.add_log(r);
            }
            Err(e) => self.add_log(format!("Error: {}", e.message())),
        };
    }

    fn fetch_remote(&mut self, remote: Option<&String>) {
        let remote = match remote {
            Some(b) => b,
            None => {
                self.add_log("remote name must not be null".to_string());
                return;
            }
        };

        let selected_repository = self.get_selected_repository();
        let repository = get_repository(&selected_repository.path).unwrap();

        match fetch_branches_repository_from_remote(remote.as_str(), &repository) {
            Ok(message) => {
                self.add_log(message);
                self.update_repository_details();
            }
            Err(e) => self.add_log(e.message().to_string()),
        }
    }

    fn checkout_to_branch(&mut self, branch_name: Option<&String>) {
        let branch_name = match branch_name {
            Some(b) => b,
            None => {
                self.add_log("Branch name must not be null".to_string());
                return;
            }
        };

        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let head = match repo.head() {
                Ok(h) => h,
                Err(_) => {
                    let sig = repo.signature().unwrap();
                    let tree_id = {
                        let mut index = repo.index().unwrap();
                        index.write_tree().unwrap()
                    };
                    let tree = repo.find_tree(tree_id).unwrap();
                    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
                    repo.head().unwrap()
                }
            };

            let oid = head.target().unwrap();
            let commit = repo.find_commit(oid).unwrap();
            let branch_name = branch_name.as_str();
            let _branch = repo.branch(branch_name, &commit, false);
            let obj = repo
                .revparse_single(&("refs/heads/".to_owned() + branch_name))
                .unwrap();
            match repo.checkout_tree(&obj, None) {
                Ok(()) => {
                    let _result = repo.set_head(&("refs/heads/".to_owned() + branch_name));
                    self.get_selected_repository().active_branch_name = branch_name.to_string();
                    self.update_repository_details();
                    self.add_log("Checkout is successful!".to_string());
                }
                Err(e) => self.add_log(format!("Error: {}", e.message())),
            };
        }
    }

    fn create_tag(&mut self, tag_name: Option<&String>) {
        let tag_name = match tag_name {
            Some(b) => b,
            None => {
                self.add_log("Tag name must not be null".to_string());
                return;
            }
        };

        if let Some(repo) = get_repository(&self.get_selected_repository().path) {
            let obj = repo
                .revparse_single(
                    &("refs/heads/".to_owned()
                        + &self.get_selected_repository().active_branch_name),
                )
                .unwrap();
            let sig = repo.signature().unwrap();
            match repo.tag(
                tag_name.as_str(),
                &obj,
                &sig,
                format!("Release {}", tag_name).as_str(),
                true,
            ) {
                Ok(_oid) => self.add_log(String::from("Tag creation is successful!")),
                Err(e) => self.add_log(format!("Error: {}", e.message())),
            };
            self.update_repository_details();
        }
    }

    fn update_repository_details(&mut self) {
        if self.selection == Selection::Repositories {
            //Get selected repository
            let rep = get_repository(&self.get_selected_repository().path);
            self.tags.unselect();
            self.tags = StatefulList::builder().items(get_repository_tags(&rep)).build();
            self.branches.unselect();
            self.branches = StatefulList::builder().items(get_repository_branches(&rep)).build();
        }
    }

    pub fn update_application_content(&mut self, path: &Path) {
        self.repositories.items.iter_mut().for_each(|f| {
            if path.to_str().unwrap().contains(&f.path.to_str().unwrap()) {
                let repository = get_repository(&f.path);
                let mut is_repository = false;
                let mut active_branch_name = String::new();

                if repository.is_some() {
                    is_repository = true;
                    active_branch_name = get_repository_active_branch(&repository);
                }

                let files_changed = get_files_changed(&repository).unwrap_or(0);
                if f.is_repository != is_repository || f.active_branch_name != active_branch_name || f.files_changed != files_changed {
                    f.set_active_branch_name(active_branch_name);
                    f.set_files_changed(files_changed);
                };
            }
        });
    }

    pub fn get_selected_repository(&mut self) -> &mut GittenRepositoryItem {
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
                }
                Err(_) => self.add_log(String::from("Could not reset!")),
            };
        }
    }

    pub fn add_log(&mut self, message: String) {
        if self.repositories.state.selected().is_some() {
            self.logs
                .items
                .push(format!("{} - {}", self.get_repository_info(), message));
        } else {
            self.logs.items.push(message);
        }

        self.logs.state.select(Some(self.logs.items.len()));
    }

    fn get_repository_info(&self) -> String {
        let r = &self.repositories.items[self.repositories.state.selected().unwrap()];
        format!("{} - {}", r.folder_name, r.active_branch_name)
    }

    pub fn generate_help(&mut self) -> String {
        match self.selection {
            Selection::Repositories => {
                if self.get_selected_repository().is_repository {
                    String::from(":co | :tag | :rh | :pull <remote> | :fetch <remote> | l to see the logs | q")
                } else {
                    String::from("No operation for non repository item | q")
                }
            }
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
        if let Err(e) = Command::new(&self.input)
            .arg(&self.get_selected_repository().path)
            .output()
        {
            self.add_log(e.to_string())
        }
    }

    pub fn scroll_logs_up(&mut self) {
        if let Some(logs) = &mut self.repository_logs {
            logs.scroll_up()
        }
    }

    pub fn scroll_logs_down(&mut self) {
        if let Some(logs) = &mut self.repository_logs {
            logs.scroll_down()
        }
    }
}

#[derive(Default)]
pub struct AppBuilder {
    pub repositories: StatefulList<GittenRepositoryItem>,
    pub branches: StatefulList<GittenStringItem>,
    pub tags: StatefulList<GittenStringItem>,
    pub path: String
}

impl AppBuilder {
    pub fn path(mut self, path: String) -> AppBuilder {
        self.path = path;
        self
    }

    fn generate_application_content(path: &String, content: &mut Vec<GittenRepositoryItem>) {
        let paths = fs::read_dir(path).unwrap();

        paths.for_each(|p| {
            let dir = p.unwrap();
            if !dir.file_name().to_str().unwrap().starts_with('.') {
                let repository = get_repository(&dir.path());
                let active_branch_name = get_repository_active_branch(&repository);
                let files_changed = get_files_changed(&repository).unwrap_or(0);

                let repository_item = GittenRepositoryItem::builder()
                    .path(fs::canonicalize(&dir.path()).unwrap())
                    .folder_name(dir.file_name().into_string().unwrap())
                    .set_is_repository(is_repository(dir.path()))
                    .files_changed(files_changed)
                    .active_branch_name(active_branch_name)
                    .build();

                content.push(repository_item);
            }
        });
        content.sort_by(|a, b| {
            a.folder_name
                .to_lowercase()
                .cmp(&b.folder_name.to_lowercase())
        });
    }

    pub fn build(self) -> App {
        let mut content = Vec::new();

        AppBuilder::generate_application_content(&self.path, &mut content);

        let (tx, rx): (
            Sender<notify::Result<Event>>,
            Receiver<notify::Result<Event>>,
        ) = channel(1);

        App {
            selection: Selection::Repositories,
            repositories: StatefulList::builder().items(content).build(),
            branches: StatefulList::builder().items(vec![]).build(),
            tags: StatefulList::builder().items(vec![]).build(),
            input: String::new(),
            input_mode: InputMode::Normal,
            logs: StatefulList::builder().items(vec![]).build(),
            repository_logs: None,
            path: self.path,
            channels: (tx, rx),
        }
    }
}
