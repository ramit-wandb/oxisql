mod connector;
mod formatter;
mod query;
mod trie;
mod ui;

use clap::Parser;
use console::Term;
use query::PROMPT;
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
    let interactive = args.execute.is_none();

    if interactive {
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
        let trie_file_path: PathBuf = {
            let ref home = std::env::var("HOME").unwrap();
            Path::new(home).join(".cache/oxisql/queries.trie.json")
        };
        let commands_trie = Trie::from_file(trie_file_path.as_path()).unwrap_or(Trie::new());

        let mut session = ui::Session::new(term, commands_trie, symbols_trie);

        loop {
            print!("{PROMPT}");
            std::io::stdout().flush().unwrap();

            println!();

            let query = session.get_user_input();

            if query.as_str() == "exit;" || query.as_str() == "quit;" {
                break;
            }

            if query.as_str() == "clear;" {
                let mut stdout = std::io::stdout();
                stdout.write("\x1B[2J\x1B[1;1H".as_bytes()).unwrap();
                continue;
            }

            session.commands_trie.insert(query.as_str());
            let start_time = Instant::now();
            let result =
                MySqlResult::parse_query(query.as_str().to_string(), connection.clone()).await;
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

        session
            .commands_trie
            .save(trie_file_path.as_path())
            .unwrap();
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
