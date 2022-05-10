use fmt::Display;
use std::{fmt, str};
use std::fmt::Error;
use std::path::PathBuf;
use git2::{Repository, RepositoryState};
use std::string::String;

pub fn is_repository(path: PathBuf) -> bool {
    match Repository::open(path) {
        Ok(_repo) => true,
        _error => false
    }
}

pub fn get_repository(path: PathBuf) -> Option<Repository>{
    match Repository::open(path) {
        Ok(repo) => Some(repo),
        Err(e) => None
    }
}

pub fn get_repository_tags(repository: Option<Repository>) -> String {
    let mut tags = String::new();
    if let Some(r) = repository {
        r.tag_names(None).iter().for_each(|f| {
            f.iter().for_each(|x| {
                if let Some(tag) = x {
                    tags = format!("{},{}", tags, tag);
                };
            });
        });
    }
    tags.trim_start_matches(',').to_string()
}