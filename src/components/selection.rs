use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(PartialEq)]
pub enum Selection {
    Repositories,
    Tags,
    Branches,
}

impl Display for Selection {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Selection::Repositories => "(R)epositories",
                Selection::Tags => "(T)ags",
                Selection::Branches => "(B)ranches",
            }
        )
    }
}