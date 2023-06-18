use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::mysql::MySqlColumn;
use sqlx::{Column, MySql, MySqlConnection, Row, TypeInfo};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MySqlTable {
    pub headers: Vec<String>,
    pub values: Vec<HashMap<String, String>>,
}

impl MySqlTable {
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
    Table(MySqlTable),
    RowsAffected(MySqlRowsAffected),
}

pub async fn get_symbols(connection: &mut MySqlConnection) -> Result<Vec<String>, sqlx::Error> {
    let mut symbols = Vec::new();
    let rows = sqlx::query("SHOW TABLES").fetch_all(connection).await?;
    for row in rows {
        symbols.push(row.try_get::<String, _>(0)?);
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
        connection: &mut MySqlConnection,
    ) -> Result<MySqlResult, sqlx::Error> {
        let first_word = query.split_whitespace().next().unwrap_or("").to_uppercase();

        if first_word.starts_with("INSERT")
            || first_word.starts_with("UPDATE")
            || first_word.starts_with("DELETE")
        {
            let affected_rows = sqlx::query(query.as_str()).execute(connection).await?;
            let result = MySqlResult::RowsAffected(MySqlRowsAffected {
                affected_rows: affected_rows.rows_affected(),
            });

            return Ok(result);
        }

        let mut result = MySqlTable::new();
        let rows = sqlx::query(query.as_str()).fetch_all(connection).await?;

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
                let value: &MySqlColumn = column.into();
                let column_idx = column.name(); // ordinal() will fix the show full processlist
                                                // bug, but better to fix it within mysql itself
                let value_str = match value.type_info().name() {
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
                row_values.insert(column_idx.to_string(), value_str);
            }
            result.values.push(row_values);
        }

        Ok(MySqlResult::Table(result))
    }
}
