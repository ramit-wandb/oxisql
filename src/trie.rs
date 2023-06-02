use std::collections::HashMap;

#[derive(Debug)]
pub struct Trie {
    children: HashMap<char, Trie>,
    word: Option<String>
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            children: HashMap::new(),
            word: None
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = self;
        for c in word.chars() {
            node = node.children.entry(c).or_insert(Trie::new());
        }
        node.word = Some(word.to_string());
    }

    pub fn search_all(&self, prefix: &str) -> Vec<String> {
        let mut node = self;
        for c in prefix.chars() {
            if let Some(child) = node.children.get(&c) {
                node = child;
            } else {
                return vec![];
            }
        }
        let mut words = vec![];
        node.traverse(&mut words);
        words
    }

    pub fn traverse(&self, words: &mut Vec<String>) {
        if let Some(word) = &self.word {
            words.push(word.clone());
        }
        for child in self.children.values() {
            child.traverse(words);
        }
    }
}
