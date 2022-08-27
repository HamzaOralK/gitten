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

/// Repository item for complex repository object
#[derive(Debug, Default)]
pub struct GittenRepositoryItem {
    pub path: PathBuf,
    pub folder_name: String,
    pub is_repository: bool,
    pub active_branch_name: String,
    pub files_changed: usize,
}

impl GittenRepositoryItem {
    pub fn builder() -> GittenRepositoryItemBuilder {
        GittenRepositoryItemBuilder::default()
    }

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

#[derive(Default)]
pub struct GittenRepositoryItemBuilder {
    pub path: PathBuf,
    pub folder_name: String,
    pub is_repository: bool,
    pub active_branch_name: String,
    pub files_changed: usize,
}

impl GittenRepositoryItemBuilder {
    pub fn path(mut self, path: PathBuf) -> GittenRepositoryItemBuilder {
        self.path = path;
        self
    }

    pub fn folder_name(mut self, folder_name: String) -> GittenRepositoryItemBuilder {
        self.folder_name = folder_name;
        self
    }

    pub fn set_is_repository(mut self, is_repository: bool) -> GittenRepositoryItemBuilder {
        self.is_repository = is_repository;
        self
    }

    pub fn active_branch_name(mut self, active_branch_name: String) -> GittenRepositoryItemBuilder {
        self.active_branch_name = active_branch_name;
        self
    }

    pub fn files_changed(mut self, files_changed: usize) -> GittenRepositoryItemBuilder {
        self.files_changed = files_changed;
        self
    }

    pub fn build(self) -> GittenRepositoryItem {
        GittenRepositoryItem {
            path: self.path,
            folder_name: self.folder_name,
            is_repository: self.is_repository,
            active_branch_name: self.active_branch_name,
            files_changed: self.files_changed
        }
    }

}

/// String item for tabs and brnaches
pub type GittenStringItem = String;

impl ConvertableToListItem for GittenStringItem {
    fn convert_to_list_item(&self, _chunk: Option<&Rect>) -> ListItem {
        ListItem::new(vec![Spans::from(vec![Span::raw(self.to_string())])])
    }
}