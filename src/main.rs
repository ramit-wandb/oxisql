use std::{pin::Pin, future::Future, collections::HashMap};
use sqlx::mysql::MySqlColumn;
use sqlx::{MySqlConnection, Connection, Row, Column};
use sqlx::TypeInfo;
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
    values: Vec<HashMap<String, String>>
}

impl MySqlQueryResult {
    fn new() -> MySqlQueryResult {
        MySqlQueryResult {
            columns: Vec::new(),
            values: Vec::new()
        }
    }

    async fn parse_query(query: String, connection: &mut MySqlConnection) -> Result<MySqlQueryResult, sqlx::Error> {
        let mut result = MySqlQueryResult::new();

        let rows = sqlx::query(query.as_str()).fetch_all(connection).await?;

        if let Some(first_row) = rows.first() {
            result.columns = first_row.columns().iter().map(|c| c.name().to_string()).collect();
        }

        for row in rows {
            let mut row_values = HashMap::new();

            for column in &result.columns {
                let column = column.as_str();
                let value : &MySqlColumn = row.column(column);
                let value_str = match value.type_info().name() {
                    "BOOLEAN" => {
                        let value : Result<bool, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "TINYINT" => {
                        let value : Result<i8, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "SMALLINT" => {
                        let value : Result<i16, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "INT" => {
                        let value : Result<i32, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "BIGINT" => {
                        let value : Result<i64, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "TINYINT UNSIGNED" => {
                        let value : Result<u8, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "SMALLINT UNSIGNED" => {
                        let value : Result<u16, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "INT UNSIGNED" => {
                        let value : Result<u32, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "BIGINT UNSIGNED" => {
                        let value : Result<u64, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "FLOAT" => {
                        let value: Result<f32, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "DOUBLE" => {
                        let value: Result<f64, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value.to_string(),
                            Err(_) => String::from("NULL")
                        }
                    },
                    "VARCHAR" | "CHAR" | "TEXT" => {
                        let value : Result<String, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => value,
                            Err(_) => String::from("NULL")
                        }
                    },
                    "VARBINARY" | "BINARY" | "BLOB" => {
                        let value : Result<Vec<u8>, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => format!("{:?}", value),
                            Err(_) => String::from("NULL")
                        }
                    },
                    // TODO: DATE, TIME, DATEIME, TIMESTAMP, JSON
                    _ => {
                        "NULL".to_string()
                    }
                };

                row_values.insert(column.to_string(), value_str);
            }

            result.values.push(row_values);
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

            if input == "clear" {
                let mut stdout = std::io::stdout();
                stdout.write("\x1B[2J\x1B[1;1H".as_bytes()).unwrap();
                continue;
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
    let size = output.len();

    // Place a line under the column names
    output.push_str("\n");
    for _ in 0..size {
        output.push_str("-");
    }
    output.push_str("\n");

    for row in &result.values {
        for column_name in &result.columns {
            let column = row.get(column_name).unwrap();
            output.push_str(column.as_str());
            output.push_str("\t");
        }
        output.push_str("\n");
    }

    output
}
