use std::{pin::Pin, future::Future};
use sqlx::{MySqlConnection, Connection};
use std::io::Write;

mod connector;
use connector::MySqlQueryResult;

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
    let connection = connection.await.expect("Could not connect to database");

    run_mysql_session(connection, mysql_args).await;
}


async fn run_mysql_session(mut connection: MySqlConnection, mysql_args: MySqlArgs) {
    let interactive = mysql_args.execute.is_none();

    if interactive {
        loop {
            print!("oxisql> ");
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if input == "exit" || input == "quit" {
                break;
            }

            if input == "clear" {
                let mut stdout = std::io::stdout();
                stdout.write("\x1B[2J\x1B[1;1H".as_bytes()).unwrap();
                continue;
            }

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
}

fn print_table(result: &MySqlQueryResult) {
    let mut max_lengths = result.headers.iter().map(|s| s.len()).collect::<Vec<usize>>();

    for row in &result.values {
        for (i, column_name) in result.headers.iter().enumerate() {
            let column = row.get(column_name).unwrap();
            max_lengths[i] = max_lengths[i].max(column.len());
        }
    }

    let mut sum_lengths: usize = max_lengths.iter().sum();
    sum_lengths += max_lengths.len() * 3 - 1;

    // Print separator
    print!("+");
    for _ in 0..sum_lengths {
        print!("-");
    }
    println!("+");

    // Print headers
    print!("| ");
    for (header, max_length) in result.headers.iter().zip(max_lengths.iter()) {
        print!("{:<width$} | ", header, width = max_length);
    }
    println!();

     // Print separator
    print!("+");
    for _ in 0..sum_lengths {
        print!("-");
    }
    println!("+"); 

    // Print rows
    for row in &result.values {
        print!("| ");
        for (i, column_name) in result.headers.iter().enumerate() {
            let column = row.get(column_name).unwrap();
            print!("{:<width$} | ", column, width = max_lengths[i]);
        }
        println!();
    }

   // Print separator
    print!("+");
    for _ in 0..sum_lengths {
        print!("-");
    }
    println!("+"); 
}
