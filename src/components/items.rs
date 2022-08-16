use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::ListItem;

pub trait ConvertableToListItem {
    fn convert_to_list_item(&self, chunk: Option<&Rect>) -> ListItem;
}

#[derive(Debug, Default)]
pub struct GittenRepositoryItem {
    pub path: PathBuf,
    pub folder_name: String,
    pub is_repository: bool,
    pub active_branch_name: String,
    pub files_changed: usize,
}

impl GittenRepositoryItem {
    pub fn set_active_branch_name(&mut self, name: String) {
        self.active_branch_name = name;
    }

    pub fn set_files_changed(&mut self, files_changed: usize) {
        self.files_changed = files_changed;
    }
}

impl Display for GittenRepositoryItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.folder_name)
    }
}

impl ConvertableToListItem for GittenRepositoryItem {
    fn convert_to_list_item(&self, chunk: Option<&Rect>) -> ListItem {
        let mut lines: Spans = Spans::default();
        let mut line_color = Color::Reset;
        if self.is_repository {
            let mut margin = 4;
            if self.files_changed > 0 {
                margin += 1;
            }
            let repeat_time = if chunk.unwrap().width
                > ((self.active_branch_name.len() as u16)
                + (self.folder_name.len() as u16)
                + margin)
            {
                chunk.unwrap().width
                    - ((self.active_branch_name.len() as u16)
                    + (self.folder_name.len() as u16)
                    + margin)
            } else {
                0
            };
            lines.0.push(Span::from(self.folder_name.clone()));
            lines.0.push(Span::from(" ".repeat((repeat_time) as usize)));
            lines.0.push(Span::raw("("));
            lines
                .0
                .push(Span::from(self.active_branch_name.to_string()));
            lines
                .0
                .push(Span::from(if self.files_changed > 0 { "*" } else { "" }));
            lines.0.push(Span::raw(")"));
            line_color = Color::Green
        } else {
            lines.0.push(Span::from(self.folder_name.clone()));
        }
        ListItem::new(lines).style(Style::default().fg(Color::White).bg(line_color))
    }
}

pub type GittenStringItem = String;

impl ConvertableToListItem for GittenStringItem {
    fn convert_to_list_item(&self, _chunk: Option<&Rect>) -> ListItem {
        ListItem::new(vec![Spans::from(vec![Span::raw(self.to_string())])])
    }
}