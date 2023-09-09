mod connector;
mod formatter;
mod query;
mod trie;

use clap::Parser;
use console::Key::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Backspace, Char, Enter, Tab};
use console::Term;
use query::Query;
use sqlx::{Connection, MySqlConnection};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use std::sync::{Arc, Mutex};

use crate::connector::{get_symbols, MySqlResult};
use crate::trie::Trie;

#[derive(Debug, Parser)]
#[command(author, version, disable_help_flag(true))]
struct MySqlArgs {
    #[arg(short, long)]
    host: String,

    #[arg(short = 'P', long, default_value = "3306")]
    port: u16,

    #[arg(short, long)]
    user: String,

    #[arg(short, long, default_value = "")]
    password: String,

    #[arg(short = 'D', long)]
    database: String,

    #[arg(short, long)]
    execute: Option<String>,
}

#[tokio::main]
async fn main() {
    let mut args: MySqlArgs = MySqlArgs::parse();
    if args.password == "" {
        args.password = rpassword::prompt_password("Password: ").expect("Could not read password");
    } else {
        eprintln!(
            "[!] Warning: password is being passed as a command line argument, this is not secure"
        )
    }

    let connection = MySqlConnection::connect(
        format!(
            "mysql://{}:{}@{}:{}/{}",
            args.user, args.password, args.host, args.port, args.database
        )
        .as_str(),
    )
    .await;

    match connection {
        Ok(connection) => {
            println!("[+] Connected to MySQL server");
            run_mysql_session(Arc::new(Mutex::new(connection)), args).await;
        }
        Err(e) => {
            eprintln!("[-] Could not connect to MySQL server: {}", e);
        }
    }
}

async fn run_mysql_session(connection: Arc<Mutex<MySqlConnection>>, args: MySqlArgs) {
    let trie_file_path: PathBuf = {
        let ref home = std::env::var("HOME").unwrap();
        Path::new(home).join(".cache/oxisql/queries.trie.json")
    };
    let interactive = args.execute.is_none();

    const PROMPT: &str = "oxisql> ";
    let term: Term = Term::stdout();

    let mut symbols_trie: Trie = Trie::new();
    println!("[+] Loading Symbols from Database");
    let symbols = get_symbols(connection.clone()).await;
    match symbols {
        Ok(symbols) => {
            symbols_trie = Trie::from_vec(symbols);
        }
        Err(e) => {
            eprintln!("[-] Could not get symbols: {}", e);
        }
    }

    if interactive {
        let mut command_trie = Trie::from_file(trie_file_path.as_path()).unwrap_or(Trie::new());

        loop {
            print!("{PROMPT}");
            std::io::stdout().flush().unwrap();

            let mut pressed_key: console::Key;
            let mut input = String::new();
            let mut command_offset: usize = 0;
            let mut commands: Vec<String> = command_trie.search_all(input.as_str());
            let mut cursor: usize = 0;
            let mut last_matched_word: String = String::new();
            let mut match_index: usize = 0;
            let mut query = Query::new();

            loop {
                pressed_key = term.read_key().unwrap();
                let (terminal_rows, terminal_cols) = term.size();

                match pressed_key {
                    Backspace => {
                        if cursor > 0 {
                            cursor -= 1;
                            input.remove(cursor);
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str())
                                .unwrap();
                        }
                        commands = command_trie.search_all(input.as_str());
                    }
                    Enter => {
                        if input.chars().last() == Some(';') {
                            break;
                        }
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
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str())
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
                            commands = command_trie.search_all(input.as_str());
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str())
                                .unwrap();
                            continue;
                        }

                        let found_command = commands.get(commands.len() - command_offset).unwrap();

                        if found_command.len() > 0 {
                            input = found_command.clone();
                            cursor = input.len();
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str())
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

                        let valid_symbols = symbols_trie.search_all(word.as_str());
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
                        if input.len() == 0 {
                            return;
                        }
                    }
                    Char(c) => {
                        input.insert(cursor, c);
                        cursor += 1;
                        commands = command_trie.search_all(input.as_str());
                        command_offset = 0;
                    }
                    _ => {
                        eprintln!("Unknown key: {:?}", pressed_key);
                    }
                }

                term.clear_line().unwrap();
                term.write_str(format!("\r{PROMPT}{input}").as_str())
                    .unwrap();
                term.move_cursor_left(input.len() - cursor).unwrap();
            }

            println!();

            let input = input.trim();
            if input == "exit;" || input == "quit;" {
                break;
            }

            if input == "clear;" {
                let mut stdout = std::io::stdout();
                stdout.write("\x1B[2J\x1B[1;1H".as_bytes()).unwrap();
                continue;
            }

            command_trie.insert(input.clone());
            let start_time = Instant::now();
            let result = MySqlResult::parse_query(input.to_string(), connection.clone()).await;
            let end_time = Instant::now();

            match result {
                Ok(value) => {
                    println!("{}", value);
                    println!(
                        "Elapsed time: {}ms",
                        end_time.duration_since(start_time).as_millis()
                    );
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        command_trie.save(trie_file_path.as_path()).unwrap();
        println!("Bye!");
    } else {
        let start_time = Instant::now();
        let result = MySqlResult::parse_query(args.execute.unwrap(), connection.clone()).await;
        let end_time = Instant::now();
        match result {
            Ok(output) => {
                println!("{}", output);
                println!(
                    "Elapsed time: {}ms",
                    end_time.duration_since(start_time).as_millis()
                );
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
