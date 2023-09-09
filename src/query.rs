use std::fmt::Display;

pub struct Query {
    pub visible_query: String,
    pub real_query: String,
}

impl Query {
    pub fn new() -> Self {
        Self {
            visible_query: String::new(),
            real_query: String::new(),
        }
    }

    pub fn handle_char(&mut self, c: char) {
        self.visible_query.push(c);
        self.real_query.push(c);
    }

    pub fn handle_enter(&mut self) {
        self.visible_query.push('\n');
    }

    pub fn handle_backspace(&mut self) {
        if self.visible_query.is_empty() {
            return;
        }

        self.visible_query.pop();
        self.real_query.pop();
    }

    pub fn set_query(&mut self, query: String) {
        self.visible_query = query.clone();
        self.real_query = query;
    }

    pub fn clear(&mut self) {
        self.visible_query.clear();
        self.real_query.clear();
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.visible_query)
    }
}
