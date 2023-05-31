
const TRIE_SIZE: usize = 256;

struct Trie {
    children: [Option<Box<Trie>>; TRIE_SIZE],
    word: Option<String>
}

impl Trie {
    fn new() -> Trie {
        Trie {
            children: [None; TRIE_SIZE],
            word: None
        }
    }

    fn insert(&mut self, word: &str) {
        let mut node = self;
        for c in word.chars() {
            let idx = c as u8 as usize;
            if node.children[idx].is_none() {
                node.children[idx] = Some(Box::new(Trie::new()));
            }
            node = node.children[idx].as_mut().unwrap();
        }
        node.word = Some(word.to_string());
    }

    fn search_all(&self, prefix: &str) -> Vec<String> {
        let mut node = self;
        for c in prefix.chars() {
            let idx = c as u8 as usize;
            if node.children[idx].is_none() {
                return vec![];
            }
            node = node.children[idx].as_ref().unwrap();
        }
        let mut words = vec![];
        node.search(&mut words);
        words
    }
}
