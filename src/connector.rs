use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::{Column, MySql, MySqlConnection, Row, TypeInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct MySqlOutputTable {
    pub headers: Vec<String>,
    pub values: Vec<HashMap<String, String>>,
}

impl MySqlOutputTable {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            values: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct MySqlRowsAffected {
    pub affected_rows: u64,
}

#[derive(Debug)]
pub enum MySqlResult {
    Table(MySqlOutputTable),
    RowsAffected(MySqlRowsAffected),
}

pub async fn get_symbols(
    connection: Arc<Mutex<MySqlConnection>>,
) -> Result<Vec<String>, sqlx::Error> {
    let mut symbols = Vec::new();
    let tables = sqlx::query("SHOW TABLES")
        .fetch_all(&mut *connection.lock().unwrap())
        .await?;
    for table in &tables {
        symbols.push(table.try_get::<String, _>(0)?);
    }

    for table in &tables {
        let columns = sqlx::query(format!("DESC {}", table.try_get::<String, _>(0)?).as_str())
            .fetch_all(&mut *connection.lock().unwrap())
            .await?;
        for column in &columns {
            symbols.push(column.try_get::<String, _>(0)?);
        }
    }

    Ok(symbols)
}

fn handle_result<T>(value: Result<Option<T>, sqlx::Error>) -> String
where
    T: ToString + sqlx::Type<MySql>,
{
    match value {
        Ok(value) => match value {
            Some(value) => value.to_string(),
            None => String::from("NULL"),
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            String::from("ERR")
        }
    }
}

impl MySqlResult {
    pub async fn parse_query(
        query: String,
        connection: Arc<Mutex<MySqlConnection>>,
    ) -> Result<Self, sqlx::Error> {
        let first_word = query.split_whitespace().next().unwrap_or("").to_uppercase();

        match first_word.as_str() {
            "INSERT" | "UPDATE" | "DELETE" | "REPLACE" => {
                let affected_rows = sqlx::query(query.as_str())
                    .execute(&mut *connection.lock().unwrap())
                    .await?;
                let result = MySqlResult::RowsAffected(MySqlRowsAffected {
                    affected_rows: affected_rows.rows_affected(),
                });
                return Ok(result);
            }
            _ => {}
        }

        let mut result = MySqlOutputTable::new();
        let rows = sqlx::query(query.as_str())
            .fetch_all(&mut *connection.lock().unwrap())
            .await?;

        if let Some(first_row) = rows.first() {
            result.headers = first_row
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect();
        }

        for row in rows {
            let mut row_values = HashMap::new();
            for column in row.columns() {
                let column_idx = column.ordinal();
                let value_str = match column.type_info().name() {
                    "BOOLEAN" => handle_result::<bool>(row.try_get(column_idx)),
                    "TINYINT" => handle_result::<i8>(row.try_get(column_idx)),
                    "SMALLINT" => handle_result::<i16>(row.try_get(column_idx)),
                    "INT" => handle_result::<i32>(row.try_get(column_idx)),
                    "BIGINT" => handle_result::<i64>(row.try_get(column_idx)),
                    "TINYINT UNSIGNED" => handle_result::<u8>(row.try_get(column_idx)),
                    "SMALLINT UNSIGNED" => handle_result::<u16>(row.try_get(column_idx)),
                    "INT UNSIGNED" => handle_result::<u32>(row.try_get(column_idx)),
                    "BIGINT UNSIGNED" => handle_result::<u64>(row.try_get(column_idx)),
                    "FLOAT" => handle_result::<f32>(row.try_get(column_idx)),
                    "DOUBLE" => handle_result::<f64>(row.try_get(column_idx)),
                    "VARCHAR" | "CHAR" | "TEXT" => handle_result::<String>(row.try_get(column_idx)),
                    "VARBINARY" | "BINARY" | "BLOB" => {
                        let value: Result<Option<Vec<u8>>, sqlx::Error> = row.try_get(column_idx);
                        match value {
                            Ok(value) => match value {
                                Some(value) => format!("{:?}", value),
                                None => String::from("NULL"),
                            },
                            Err(_) => String::from("ERR"),
                        }
                    }
                    "DATETIME" => handle_result::<NaiveDateTime>(row.try_get(column_idx)),
                    "DATE" => handle_result::<NaiveDate>(row.try_get(column_idx)),
                    "TIME" => handle_result::<NaiveTime>(row.try_get(column_idx)),
                    "TIMESTAMP" => handle_result::<DateTime<Utc>>(row.try_get(column_idx)),
                    "JSON" => handle_result::<serde_json::Value>(row.try_get(column_idx)),
                    _ => "NULL".to_string(),
                };
                row_values.insert(column.name().to_string(), value_str);
            }
            result.values.push(row_values);
        }

        Ok(MySqlResult::Table(result))
    }
}
