use std::fmt::Display;
use std::path::PathBuf;
use git2::{Repository};
use tui::text::{Span, Spans};
use tui::widgets::ListItem;

pub fn is_repository(path: PathBuf) -> bool {
    match Repository::open(path) {
        Ok(_repo) => true,
        _error => false
    }
}

pub fn get_repository(path: PathBuf) -> Option<Repository>{
    match Repository::open(path) {
        Ok(repo) => Some(repo),
        Err(_e) => None
    }
}

pub fn get_repository_tags(repository: &Option<Repository>) -> Vec<ListItem> {
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
    convert_to_list_item(tags)
}

pub fn get_repository_branches(repository: &Option<Repository>) -> Vec<ListItem> {
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
    convert_to_list_item(branches_string)
}

fn convert_to_list_item<T: Display>(iterator: Vec<T>) -> Vec<ListItem<'static>> {
    iterator.iter()
        .rev()
        .map(|f| {
            ListItem::new(vec![
                Spans::from(vec![
                    Span::raw(format!("{}", f))
                ])
            ])
        })
        .collect()
}