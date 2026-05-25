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
    /// Most recent large paste; expanded back into the message on submit.
    pub pasted_buffer: Option<String>,
    /// (command, description) pairs matching the slash prefix in the buffer.
    pub slash_matches: Vec<(String, String)>,
    pub slash_selected: usize,
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
            pasted_buffer: None,
            slash_matches: Vec::new(),
            slash_selected: 0,
        }
    }

    /// Expand the buffer for submission: if a large paste was made, replace
    /// the `[Pasted …]` token with the real content.
    pub fn expand_for_submit(&self) -> String {
        if let Some(real) = &self.pasted_buffer {
            if let Some(start) = self.buffer.find("[Pasted ") {
                if let Some(rel_end) = self.buffer[start..].find(']') {
                    let end = start + rel_end + 1;
                    let mut out = String::with_capacity(self.buffer.len() + real.len());
                    out.push_str(&self.buffer[..start]);
                    out.push_str(real);
                    out.push_str(&self.buffer[end..]);
                    return out;
                }
            }
        }
        self.buffer.clone()
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
        self.pasted_buffer = None;
    }

    pub fn get_display_text(&self) -> &str { &self.buffer }

    /// (line_index, col_index) for the current cursor byte position.
    pub fn cursor_line_col(&self) -> (usize, usize) {
        let mut line = 0;
        let mut last_nl = 0;
        for (i, b) in self.buffer.as_bytes().iter().enumerate() {
            if i >= self.cursor_pos { break; }
            if *b == b'\n' {
                line += 1;
                last_nl = i + 1;
            }
        }
        let col = self.buffer[last_nl..self.cursor_pos.min(self.buffer.len())]
            .chars()
            .count();
        (line, col)
    }

    pub fn line_count(&self) -> usize {
        self.buffer.lines().count().max(1)
            + if self.buffer.ends_with('\n') { 1 } else { 0 }
    }

    pub fn cursor_up_line(&mut self) -> bool {
        let (line, col) = self.cursor_line_col();
        if line == 0 { return false; }
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        let target_line = line - 1;
        let target_col = col.min(lines[target_line].chars().count());
        let mut pos = 0usize;
        for l in lines.iter().take(target_line) {
            pos += l.len() + 1;
        }
        pos += lines[target_line]
            .char_indices()
            .nth(target_col)
            .map(|(i, _)| i)
            .unwrap_or_else(|| lines[target_line].len());
        self.cursor_pos = pos;
        true
    }

    pub fn cursor_down_line(&mut self) -> bool {
        let (line, col) = self.cursor_line_col();
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        if line + 1 >= lines.len() { return false; }
        let target_line = line + 1;
        let target_col = col.min(lines[target_line].chars().count());
        let mut pos = 0usize;
        for l in lines.iter().take(target_line) {
            pos += l.len() + 1;
        }
        pos += lines[target_line]
            .char_indices()
            .nth(target_col)
            .map(|(i, _)| i)
            .unwrap_or_else(|| lines[target_line].len());
        self.cursor_pos = pos;
        true
    }
}

impl Default for InputState {
    fn default() -> Self { Self::new() }
}
