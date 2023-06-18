use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Trie {
    root: TrieNode,
    index : usize
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            root: TrieNode::new(),
            index: 0
        }
    }

    pub fn from_file(filename: &Path) -> Option<Trie> {
        let file = File::open(filename);
        if let Ok(file) = file {
            let trie = serde_json::from_reader(file);
            if let Ok(trie) = trie {
                return Some(trie);
            }
        }

        return None;
    }

    pub fn save(&self, filename: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // create all parent directories
        std::fs::create_dir_all(filename.parent().unwrap());
        let file = File::create(filename).expect("Unable to create file");
        serde_json::to_writer(file, self).expect("Unable to write data");
        Ok(())
    }

    pub fn insert(&mut self, word: &str) {
        self.root.insert(word, self.index);
        self.index += 1;
    }

    pub fn search_all(&self, prefix: &str) -> Vec<String> {
        let mut vector = self.root.search_all(prefix);

        // Sort by index
        vector.sort_by(|a, b| {
            let a_index = self.get_index(a).unwrap();
            let b_index = self.get_index(b).unwrap();
            a_index.cmp(&b_index)
        });

        vector
    }

    fn get_index(&self, word: &str) -> Option<usize> {
        let mut node = &self.root;
        for c in word.chars() {
            if let Some(child) = node.children.get(&c) {
                node = child;
            } else {
                return None;
            }
        }
        node.index
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    word: Option<String>,
    index: Option<usize>
}

impl TrieNode {
    fn new() -> TrieNode {
        TrieNode {
            children: HashMap::new(),
            word: None,
            index: None
        }
    }

    fn insert(&mut self, word: &str, index: usize) {
        let mut node = self;
        for c in word.chars() {
            node = node.children.entry(c).or_insert(TrieNode::new());
        }
        node.word = Some(word.to_string());
        node.index = Some(index);
    }

    fn search_all(&self, prefix: &str) -> Vec<String> {
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
        words.clone()
    }

    fn traverse(&self, words: &mut Vec<String>) {
        if let Some(word) = &self.word {
            words.push(word.clone());
        }
        for child in self.children.values() {
            child.traverse(words);
        }
    }
}
