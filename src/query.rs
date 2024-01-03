use std::fmt::Display;

pub struct Query {
    pub visible_query: String,
    pub real_query: String,
}

pub const PROMPT: &str = "oxisql>";

impl Query {
    pub fn new() -> Self {
        Self {
            visible_query: String::new(),
            real_query: String::new(),
        }
    }

    pub fn handle_char(&mut self, cursor_position: usize, c: char) {
        self.visible_query.insert(cursor_position, c);
        self.real_query.insert(cursor_position, c);
    }

    pub fn handle_enter(&mut self) {
        self.visible_query.push('\n');
    }

    pub fn handle_backspace(&mut self, cursor_position: usize) {
        if self.visible_query.is_empty() {
            return;
        }

        self.visible_query.remove(cursor_position);
        self.real_query.remove(cursor_position);
    }

    pub fn set_query(&mut self, query: String) {
        self.visible_query = query.clone();
        self.real_query = query;
    }

    pub fn clear(&mut self) {
        self.visible_query.clear();
        self.real_query.clear();
    }

    pub fn len(&self) -> usize {
        self.visible_query.len()
    }

    pub fn chars(&self) -> std::str::Chars<'_> {
        self.visible_query.chars()
    }

    pub fn as_str(&self) -> &str {
        self.real_query.as_str()
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.visible_query)
    }
}
