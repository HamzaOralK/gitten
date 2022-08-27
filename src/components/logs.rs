#[derive(Clone)]
pub struct Logs {
    pub logs: String,
    pub offset: (u16, u16)
}

impl Logs {
    pub fn builder() -> LogsBuilder {
        LogsBuilder::default()
    }

    pub fn scroll_down(&mut self) {
        self.offset.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        if self.offset.0 > 0 {
            self.offset.0 -= 1;
        }
    }
}

#[derive(Default)]
pub struct LogsBuilder {
    pub logs: String
}

impl LogsBuilder {
    pub fn logs(mut self, logs: String) -> LogsBuilder {
        self.logs = logs;
        self
    }

    pub fn build(self) -> Logs {
        Logs {
            logs: self.logs,
            offset: (0, 0)
        }
    }
}