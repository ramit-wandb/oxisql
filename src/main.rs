use std::{pin::Pin, future::Future, collections::HashMap};
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

#[derive(Debug)]
struct MySqlQueryResult {
    columns: Vec<String>,
    values: HashMap<String, Vec<String>>
}

impl MySqlQueryResult {
    fn new() -> MySqlQueryResult {
        MySqlQueryResult {
            columns: Vec::new(),
            values: HashMap::new()
        }
    }

    async fn parse_query(query: String,connection: &mut MySqlConnection) -> Result<MySqlQueryResult, sqlx::Error> {
        let mut result = MySqlQueryResult::new();

        let mut rows = sqlx::query(query.as_str()).fetch(connection);
        let mut columns_read = false;

        while let Some(row) = rows.next().await {
            let row = row?;
            if !columns_read {
                result.columns = row.columns().iter()
                    .map(|c| c.name())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(); 
                columns_read = true;
            }

            for column_name in &result.columns {
                let resp = row.try_get(&column_name as &str);
                let values = result.values.entry(column_name.clone()).or_insert(Vec::new());
                match resp {
                    Ok(value) => {
                        values.push(value);
                    },
                    Err(_) => {
                        values.push(String::from("NULL"));
                    }
                }
            }
        }

        Ok(result)
    }
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

            let result = MySqlQueryResult::parse_query(input.to_string(), &mut connection).await;

            match result {
                Ok(value) => { 
                    println!("{}", format_result(value));
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    } else {
        let result = MySqlQueryResult::parse_query(mysql_args.execute.unwrap(), &mut connection).await;
        match result {
            Ok(value) => { 
                println!("{}", format_result(value));
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn format_result(result: MySqlQueryResult) -> String {
    let mut output = String::new();

    for column in &result.columns {
        output.push_str(column.as_str());
        output.push_str("\t");
    }
    output.push_str("\n");

    for column_name in &result.columns {
        let column = result.values.get(column_name).unwrap();
        for value in column {
            output.push_str(value.as_str());
            output.push_str("\t");
        }
    }

    output
}
