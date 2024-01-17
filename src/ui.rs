use std::path::PathBuf;

use crate::query::{Query, PROMPT};
use crate::trie::Trie;

use console::Key::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Backspace, Char, Enter, Tab};
use console::Term;

pub struct Session {
    terminal: Term,
    pub commands_trie: Trie,
    symbols_trie: Trie,
}

impl Session {
    pub fn new(terminal: Term, commands_trie: Trie, symbols_trie: Trie) -> Self {
        Self {
            terminal,
            commands_trie,
            symbols_trie,
        }
    }

    pub fn get_user_input(&self) -> Query {
        // TODO: Result
        let mut query = Query::new();
        let mut pressed_key: console::Key;
        let mut command_offset: usize = 0;
        let mut input = String::new();
        let mut cursor = 0;
        let mut commands: Vec<String> = self.commands_trie.search_all(input.as_str());
        let mut last_matched_word: String = String::new();
        let mut match_index: usize = 0;

        loop {
            pressed_key = self.terminal.read_key().unwrap(); // TODO: Remove unwrap
                                                             //let (terminal_rows, terminal_cols) = self.terminal.size();

            match pressed_key {
                Backspace => {
                    if cursor > 0 {
                        cursor -= 1;
                        query.handle_backspace(cursor);
                        self.terminal.clear_line().unwrap();
                        self.terminal
                            .write_str(format!("\r{PROMPT}{input}").as_str())
                            .unwrap();
                    }
                    commands = self.commands_trie.search_all(input.as_str());
                }
                Enter => {
                    if query.chars().last() == Some(';') {
                        break;
                    }
                    query.handle_enter();
                }
                ArrowUp => {
                    if commands.len() == 0 {
                        continue;
                    }
                    if command_offset != commands.len() {
                        command_offset += 1;
                    }

                    let found_command = commands.get(commands.len() - command_offset).unwrap();

                    if found_command.len() > 0 {
                        input = found_command.clone();
                        cursor = input.len();
                        self.terminal.clear_line().unwrap();
                        self.terminal
                            .write_str(format!("\r{PROMPT}{input}").as_str())
                            .unwrap();
                    }
                }
                ArrowDown => {
                    if commands.len() == 0 {
                        continue;
                    }
                    if command_offset >= 1 {
                        command_offset -= 1;
                    }

                    if command_offset == 0 {
                        input = String::new();
                        cursor = 0;
                        commands = self.commands_trie.search_all(input.as_str());
                        self.terminal.clear_line().unwrap();
                        self.terminal
                            .write_str(format!("\r{PROMPT}{input}").as_str())
                            .unwrap();
                        continue;
                    }

                    let found_command = commands.get(commands.len() - command_offset).unwrap();

                    if found_command.len() > 0 {
                        input = found_command.clone();
                        cursor = input.len();
                        self.terminal.clear_line().unwrap();
                        self.terminal
                            .write_str(format!("\r{PROMPT}{input}").as_str())
                            .unwrap();
                    }
                }
                ArrowLeft => {
                    cursor = if cursor > 0 { cursor - 1 } else { 0 };
                }
                ArrowRight => {
                    cursor = if cursor < input.len() {
                        cursor + 1
                    } else {
                        input.len()
                    };
                }
                Tab => {
                    // Copy string till last space or start of string
                    let word = input[..cursor]
                        .chars()
                        .rev()
                        .take_while(|c| *c != ' ')
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>();

                    let valid_symbols = self.symbols_trie.search_all(word.as_str());
                    if valid_symbols.len() == 0 {
                        continue;
                    }

                    if last_matched_word == word {
                        // Concatenate matched symbol to input
                        input = input[..cursor - word.len()].to_string()
                            + valid_symbols.get(match_index).unwrap();
                        match_index = (match_index + 1) % valid_symbols.len();
                    } else {
                        match_index = 0;
                    }

                    last_matched_word = word.clone();
                }
                Char('\u{4}') => {
                    // ctrl-d
                    if query.len() == 0 {
                        // Exit process
                        std::process::exit(0); // TODO, Return this in an error enum. This function
                                               // should not be responsible for exiting the process
                    }
                }
                Char(c) => {
                    query.handle_char(cursor, c);
                    cursor += 1;
                    commands = self.commands_trie.search_all(query.as_str());
                    command_offset = 0;
                }
                _ => {
                    eprintln!("Unknown key: {:?}", pressed_key);
                }
            }

            self.terminal.clear_line().unwrap();
            self.terminal
                .write_str(format!("\r{PROMPT}{query}").as_str())
                .unwrap();
            self.terminal
                .move_cursor_left(query.len() - cursor)
                .unwrap();
        }

        return query;
    }
}
