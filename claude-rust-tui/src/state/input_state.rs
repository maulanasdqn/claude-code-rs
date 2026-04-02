#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Insert,
    Normal,
}

pub struct InputState {
    pub buffer: String,
    pub cursor_pos: usize,
    pub mode: InputMode,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub suggestion: String,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
            mode: InputMode::Insert,
            history: Vec::new(),
            history_index: None,
            suggestion: String::new(),
        }
    }

    pub fn complete_suggestion(&mut self) {
        if !self.suggestion.is_empty() {
            self.buffer.push_str(&self.suggestion);
            self.cursor_pos = self.buffer.len();
            self.suggestion.clear();
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_pos >= self.buffer.len() {
            self.buffer.push(c);
        } else {
            self.buffer.insert(self.cursor_pos, c);
        }
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            let prev = self.buffer[..self.cursor_pos]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.buffer.drain(prev..self.cursor_pos);
            self.cursor_pos = prev;
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            let next = self.buffer[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.buffer.len());
            self.buffer.drain(self.cursor_pos..next);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = self.buffer[..self.cursor_pos]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.cursor_pos = self.buffer[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.buffer.len());
        }
    }

    pub fn move_word_left(&mut self) {
        let s = &self.buffer[..self.cursor_pos];
        let trimmed = s.trim_end_matches(|c: char| !c.is_alphanumeric());
        let word_end = trimmed.rfind(|c: char| !c.is_alphanumeric()).map(|i| i + 1).unwrap_or(0);
        self.cursor_pos = word_end;
    }

    pub fn move_word_right(&mut self) {
        let s = &self.buffer[self.cursor_pos..];
        let skip = s.find(|c: char| c.is_alphanumeric()).unwrap_or(s.len());
        let after = &s[skip..];
        let word_end = after.find(|c: char| !c.is_alphanumeric()).unwrap_or(after.len());
        self.cursor_pos = (self.cursor_pos + skip + word_end).min(self.buffer.len());
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() { return; }
        let idx = match self.history_index {
            None => self.history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
        };
        self.history_index = Some(idx);
        self.buffer = self.history[idx].clone();
        self.cursor_pos = self.buffer.len();
    }

    pub fn history_next(&mut self) {
        match self.history_index {
            None => {}
            Some(i) if i + 1 < self.history.len() => {
                let idx = i + 1;
                self.history_index = Some(idx);
                self.buffer = self.history[idx].clone();
                self.cursor_pos = self.buffer.len();
            }
            _ => { self.history_index = None; self.buffer.clear(); self.cursor_pos = 0; }
        }
    }

    pub fn push_history(&mut self, text: String) {
        if !text.is_empty() && self.history.last().map(|s| s.as_str()) != Some(&text) {
            self.history.push(text);
        }
        self.history_index = None;
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
        self.history_index = None;
    }

    pub fn get_display_text(&self) -> &str { &self.buffer }
}

impl Default for InputState {
    fn default() -> Self { Self::new() }
}
