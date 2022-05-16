use std::fmt::Display;
use std::path::PathBuf;
use git2::{Repository};
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem};
use crate::App;
use crate::app::{AlfredRepository, Selection};

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

pub fn convert_vector_to_list_item_vector<T: Display>(iterator: &Vec<T>) -> Vec<ListItem<'static>> {
    iterator.iter()
        .map(|f| {
            ListItem::new(vec![
                Spans::from(vec![
                    Span::raw(format!("{}", f))
                ])
            ])
        })
        .collect()
}

pub fn create_selection_list_from_vector<'a, T: Display>(v: &'a Vec<T>, b: Block<'a>) -> List<'a > {
    List::new(convert_vector_to_list_item_vector(v))
        .block(b)
        .highlight_style(
            Style::default().add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ")
}

pub fn convert_alfred_repository_to_list_item<'a>(item: &'a AlfredRepository, chunk: &'a Rect) -> ListItem<'a> {
    let mut lines: Spans = Spans::default();
    let mut line_color = Color::Reset;
    if item.is_repository {
        let repeat_time = if chunk.width > ((item.active_branch_name.len() as u16) + (item.folder_name.len() as u16) + 6) {
            chunk.width - ((item.active_branch_name.len() as u16) + (item.folder_name.len() as u16) + 6)
        } else {
            0
        };
        lines.0.push(Span::from(item.folder_name.clone()));
        lines.0.push(Span::from(" ".repeat((repeat_time) as usize)));
        lines.0.push(Span::raw("("));
        lines.0.push(Span::from(item.active_branch_name.to_string()));
        lines.0.push(Span::raw(")"));
        line_color = Color::Green
    } else {
        lines.0.push(Span::from(item.folder_name.clone()));
    }
    ListItem::new(lines).style(Style::default().fg(Color::White).bg(line_color))
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
