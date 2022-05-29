use std::fmt::{Display};
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem};
use crate::App;
use crate::app::{ConvertableToListItem, Selection};

pub fn convert_vector_to_list_item_vector<'a, T: Display + ConvertableToListItem>(iterator: &'a [T], r: Option<&'a Rect>) -> Vec<ListItem<'a>> {
    iterator.iter()
        .map(|f| {
            f.convert_to_list_item(r)
        })
        .collect()
}

pub fn create_selection_list_from_vector<'a, T: Display + ConvertableToListItem>(v: &'a [T], b: Block<'a>, r: Option<&'a Rect>) -> List<'a > {
    List::new(convert_vector_to_list_item_vector(v, r))
        .block(b)
        .highlight_style(
            Style::default().add_modifier(Modifier::BOLD).bg(Color::Blue),
        )
}

pub fn create_block_with_selection(app: &App, selection: Selection) -> Block<'static> {
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

pub fn create_block_with_title(title: &str) -> Block<'static> {
    let b = Block::default();
    b.borders(Borders::ALL)
        .title(Spans::from(vec![
            Span::styled(title.to_string(), Style::default().bg(Color::Black).fg(Color::White))
        ]))
}

pub fn create_block() -> Block<'static> {
    let b = Block::default();
    b.borders(Borders::NONE)
}
