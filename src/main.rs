mod connector;
mod formatter;
mod trie;

use sqlx::{MySqlConnection, Connection};
use std::io::Write;
use console::Term;
use console::Key::{Char, Backspace, Enter, ArrowUp, ArrowDown, ArrowLeft, ArrowRight};
use clap::Parser;

use connector::MySqlQueryResult;
use formatter::print_table;
use trie::Trie;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    disable_help_flag(true)
)]
struct MySqlArgs {
    #[arg(short, long)]
    host: String,

    #[arg(short='P', long, default_value="3306")]
    port: u16,

    #[arg(short, long)]
    user: String,

    #[arg(short, long, default_value="")]
    password: String,

    #[arg(short='D', long)]
    database: String,

    #[arg(short, long)]
    execute: Option<String>
}

#[tokio::main]
async fn main() {
    let mut args: MySqlArgs = MySqlArgs::parse();
    if args.password == "" {
        args.password = rpassword::prompt_password("Password: ").expect("Could not read password");
    } else {
        eprintln!("[!] Warning: password is being passed as a command line argument, this is not secure")
    }

    let connection = MySqlConnection::connect(
        format!(
            "mysql://{}:{}@{}:{}/{}",
            args.user,
            args.password,
            args.host,
            args.port,
            args.database
        ).as_str()
    ).await;

    match connection {
        Ok(connection) => {
            println!("[+] Connected to MySQL server");
            run_mysql_session(connection, args).await;
        },
        Err(e) => {
            eprintln!("[-] Could not connect to MySQL server: {}", e);
        }
    }
}

const TRIE_FILE_PATH: &str = "~/.oxisql/trie.json";
async fn run_mysql_session(mut connection: MySqlConnection, args: MySqlArgs) {
    let interactive = args.execute.is_none();
    const PROMPT: &str = "oxisql> ";
    let term : Term = Term::stdout();

    if interactive {
        let mut command_trie = Trie::from_file(TRIE_FILE_PATH).unwrap_or(Trie::new());

        loop {
            print!("{PROMPT}");
            std::io::stdout().flush().unwrap();

            let mut pressed_key: console::Key;
            let mut input = String::new();
            let mut command_offset : usize = 0;
            let mut commands : Vec<String> = command_trie.search_all(input.as_str());
            let mut cursor : usize = 0;

            loop {
                pressed_key = term.read_key().unwrap();

                match pressed_key {
                    Backspace  => {
                        if cursor > 0 {
                            cursor -= 1;
                            input.remove(cursor);
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                        }
                        commands = command_trie.search_all(input.as_str());
                        term.clear_line().unwrap();
                        term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                    },
                    Enter => {
                        if input.chars().last() == Some(';') {
                            break;
                        } 
                    },
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
                            term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                        }
                    },
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
                            term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                            continue;
                        }

                        let found_command = commands.get(commands.len() - command_offset).unwrap();
                            
                        if found_command.len() > 0 {
                            input = found_command.clone();
                            cursor = input.len();
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                        }
                    },
                    ArrowLeft => {
                        cursor = if cursor > 0 { cursor - 1 } else { 0 };
                    },
                    ArrowRight => {
                        cursor = if cursor < input.len() { cursor + 1 } else { input.len() };
                    },
                    Char(c) => {
                        input.insert(cursor, c);
                        cursor += 1;
                        commands = command_trie.search_all(input.as_str());
                        command_offset = 0;
                    },
                    _ => {
                        eprintln!("Unknown key: {:?}", pressed_key);
                    }
                }

                term.clear_line().unwrap();
                term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
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
            let result = MySqlQueryResult::parse_query(input.to_string(), &mut connection).await;

            match result {
                Ok(value) => { 
                    print_table(&value);
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        command_trie.save(TRIE_FILE_PATH).unwrap();
        println!("Bye!");
    } else {
        let result = MySqlQueryResult::parse_query(args.execute.unwrap(), &mut connection).await;
        match result {
            Ok(value) => {
                print_table(&value)
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    connection.close().await.expect("Could not close connection");
}
