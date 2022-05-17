use std::fmt::Display;
use std::path::PathBuf;
use git2::{Cred, CredentialType, Repository};
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem};
use crate::App;
use crate::app::{ConvertableToListItem, Selection};

pub fn is_repository(path: PathBuf) -> bool {
    match Repository::open(path) {
        Ok(_repo) => true,
        _error => false
    }
}

pub fn get_repository(path: &String) -> Option<Repository> {
    match Repository::open(path) {
        Ok(repo) => Some(repo),
        Err(_e) => None
    }
}

pub fn get_repository_tags(repository: &Option<Repository>) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(r) = repository {
        r.tag_names(Some("[0-99999999].[0-99999999].[0-99999999]")).iter().for_each(|f| {
            f.iter().for_each(|x| {
                if let Some(tag) = x {
                    tags.push(tag.to_string());
                };
            });
        });
    }
    tags
}

pub fn get_repository_branches(repository: &Option<Repository>) -> Vec<String> {
    let mut branches_string = Vec::new();

    if let Some(r) = repository {
        let branches = match r.branches(None) {
            Ok(branches) => Some(branches),
            Err(_) => None
        };

        branches.unwrap().for_each(|b| {
            let b1 = b.unwrap().0.name().unwrap().unwrap().to_string();
            branches_string.push(b1);
        });
    }
    branches_string
}

pub fn get_repository_active_branch(repository: &Option<Repository>) -> String {
    let mut branch_id: String = "".to_string();
    if let Some(r) = repository {
        branch_id = r.head().unwrap().name().unwrap().replace("refs/heads/", "").to_string()
    }
    branch_id
}

pub fn convert_vector_to_list_item_vector<'a, T: Display + ConvertableToListItem>(iterator: &'a Vec<T>, r: Option<&'a Rect>) -> Vec<ListItem<'a>> {
    iterator.iter()
        .map(|f| {
            f.convert_to_list_item(r)
        })
        .collect()
}

pub fn create_selection_list_from_vector<'a, T: Display + ConvertableToListItem>(v: &'a Vec<T>, b: Block<'a>, r: Option<&'a Rect>) -> List<'a > {
    List::new(convert_vector_to_list_item_vector(v, r))
        .block(b)
        .highlight_style(
            Style::default().add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
}

pub fn create_block_with_title(app: &App, selection: Selection) -> Block<'static> {
    let b = Block::default();

    let style = if app.selection == selection {
        Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::Black).fg(Color::White)
    };

    b.borders(Borders::ALL)
        .title(Spans::from(vec![
            Span::styled(selection.to_string(), style)
        ]))
}

pub fn create_block() -> Block<'static> {
    let b = Block::default();
    b.borders(Borders::NONE)
}

pub fn git_credentials_callback(
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