use std::{pin::Pin, future::Future};
use sqlx::{MySqlConnection, Connection, Row, Column};
use futures::StreamExt;
use std::io::Write;

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

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mysql_args: MySqlArgs = MySqlArgs::parse_args(args);

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
            print!("mysql> ");
            // Flush stdout so that the prompt is printed
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if input == "exit" {
                break;
            }
            let mut rows = sqlx::query(input).fetch(&mut connection);
            while let Some(row) = rows.next().await {
                let row = row.expect("Could not fetch row");
                let column_names = row.columns().iter().map(|c| c.name()).collect::<Vec<_>>(); 
                for column_name in column_names {
                    let resp : Result<String, sqlx::Error>= row.try_get(column_name);
                    match resp {
                        Ok(value) => print!("{} : {}\n", column_name, value),
                        Err(_) => print!("{} : NULL\n", column_name)
                    }
                } 
                println!();
            }
        }
    } else {
        todo!("Non-interactive mode not implemented yet");
    }

}
