use std::{pin::Pin, future::Future};
use sqlx::{MySqlConnection, Connection};
use std::io::Write;
use console::Term;
use console::Key::{Char, Backspace, Enter, ArrowUp, ArrowDown};

mod connector;
use connector::MySqlQueryResult;

mod formatter;
use formatter::print_table;

mod trie;
use trie::Trie;

#[derive(Debug)]
struct MySqlArgs {
    host: String,
    port: u16,
    user: String,
    password: String,
    database: String,
    execute: Option<String>
}

impl MySqlArgs {
    fn new() -> MySqlArgs {
        MySqlArgs { 
            host: String::from(""), 
            port: 0,
            user: String::from(""),
            password: String::from(""),
            database: String::from(""),
            execute: None
        }
    }

    fn parse_args(args: Vec<String>) -> MySqlArgs {
        let mut opts = MySqlArgs::new();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--host" => {
                    i += 1;
                    opts.host = args[i].clone();
                },
                "-P" | "--port" => {
                    i += 1;
                    opts.port = args[i].parse::<u16>().expect("Invalid port");
                },
                "-u" | "--user" => {
                    i += 1;
                    opts.user = args[i].clone();
                },
                "-p" | "--password" => {
                    i += 1;
                    opts.password = args[i].clone();
                },
                "-D" | "--database" => {
                    i += 1;
                    opts.database = args[i].clone();
                },
                "-e" | "--execute" => {
                    i += 1;
                    opts.execute = Some(args[i].clone());
                },
                _ => {
                    println!("Unknown argument: {}", args[i]);
                }
            }
            i += 1;
        }

        opts
    }
}

const HELP: &str = "Usage: oxisql -h <host> -P <port> -u <user> -p <password> -D <database> [-e <query>]";

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("{HELP}");
        return;
    }

    let mysql_args: MySqlArgs = MySqlArgs::parse_args(args);

    if mysql_args.host == "" || mysql_args.port == 0 || mysql_args.user == "" || mysql_args.password == "" || mysql_args.database == "" {
        println!("{HELP}");
        return;
    }

    let connection : Pin<Box<dyn Future<Output = Result<MySqlConnection, sqlx::Error>>>> = MySqlConnection::connect(
        format!(
            "mysql://{}:{}@{}:{}/{}",
            mysql_args.user,
            mysql_args.password,
            mysql_args.host,
            mysql_args.port,
            mysql_args.database
        ).as_str()
    ); 
    let connection : MySqlConnection = connection.await.expect("Could not connect to database");

    run_mysql_session(connection, mysql_args).await;
}


async fn run_mysql_session(mut connection: MySqlConnection, mysql_args: MySqlArgs) {
    let interactive = mysql_args.execute.is_none();
    let mut command_trie = Trie::new();

    const PROMPT: &str = "oxisql> ";
    let term : Term = Term::stdout();

    if interactive {
        loop {
            print!("{PROMPT}");
            std::io::stdout().flush().unwrap();

            let mut pressed_key: console::Key;
            let mut input = String::new();
            let mut command_offset : usize = 0;
            let mut commands : Vec<String> = command_trie.search_all(input.as_str()).clone();

            loop {
                pressed_key = term.read_key().unwrap();

                match pressed_key {
                    Backspace  => {
                        input.pop();
                        commands = command_trie.search_all(input.as_str()).clone();
                        term.clear_line().unwrap();
                        term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                    },
                    Enter => {
                        if input.chars().last() == Some(';') {
                            break
                        } else {
                            term.clear_line().unwrap();
                            term.write_str("\n{PROMPT}").unwrap();
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
                            commands = command_trie.search_all(input.as_str()).clone();
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                            continue;
                        }

                        let found_command = commands.get(commands.len() - command_offset).unwrap();
                            
                        if found_command.len() > 0 {
                            input = found_command.clone();
                            term.clear_line().unwrap();
                            term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
                        }
                    }
                    Char(c) => {
                        input.push(c);
                        commands = command_trie.search_all(input.as_str()).clone();
                        command_offset = 0;
                    },
                    _ => {
                        eprintln!("Unknown key: {:?}", pressed_key);
                    }
                }

                term.clear_line().unwrap();
                term.write_str(format!("\r{PROMPT}{input}").as_str()).unwrap();
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
    } else {
        let result = MySqlQueryResult::parse_query(mysql_args.execute.unwrap(), &mut connection).await;
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
