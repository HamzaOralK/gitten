#[derive(Clone)]
pub struct Logs {
    pub log: String,
    pub offset: (u16, u16)
}

impl Logs {
    pub fn scroll_down(&mut self) {
        self.offset.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        if self.offset.0 > 0 {
            self.offset.0 -= 1;
        }
    }
}