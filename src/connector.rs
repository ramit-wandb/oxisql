use std::collections::HashMap;
use sqlx::{MySqlConnection, Row, Column, TypeInfo, MySql};
use sqlx::mysql::MySqlColumn;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, DateTime, Utc};

#[derive(Debug)]
pub struct MySqlQueryResult {
    pub headers: Vec<String>,
    pub values: Vec<HashMap<String, String>>
}

fn handle_result<T: ToString + sqlx::Type<MySql>>(value: Result<Option<T>, sqlx::Error>) -> String {
    match value {
        Ok(value) => match value {
            Some(value) => value.to_string(),
            None => String::from("NULL")
        },
        Err(_) => String::from("NULL")
    }
}

impl MySqlQueryResult {
    fn new() -> MySqlQueryResult {
        MySqlQueryResult {
            headers: Vec::new(),
            values: Vec::new()
        }
    } 

    pub async fn parse_query(query: String, connection: &mut MySqlConnection) -> Result<MySqlQueryResult, sqlx::Error> {
        let mut result = MySqlQueryResult::new();

        let rows = sqlx::query(query.as_str()).fetch_all(connection).await?;

        if let Some(first_row) = rows.first() {
            result.headers = first_row.columns().iter().map(|c| c.name().to_string()).collect();
        }

        for row in rows {
            let mut row_values = HashMap::new();
            for column in &result.headers {
                let column = column.as_str();
                let value : &MySqlColumn = row.column(column);

                let value_str = match value.type_info().name() {
                    "BOOLEAN" => handle_result::<bool>(row.try_get(column)),
                    "TINYINT" => handle_result::<i8>(row.try_get(column)),
                    "SMALLINT" => handle_result::<i16>(row.try_get(column)), 
                    "INT" => handle_result::<i32>(row.try_get(column)),
                    "BIGINT" => handle_result::<i64>(row.try_get(column)),
                    "TINYINT UNSIGNED" => handle_result::<u8>(row.try_get(column)),
                    "SMALLINT UNSIGNED" => handle_result::<u16>(row.try_get(column)),
                    "INT UNSIGNED" => handle_result::<u32>(row.try_get(column)),
                    "BIGINT UNSIGNED" => handle_result::<u64>(row.try_get(column)),
                    "FLOAT" => handle_result::<f32>(row.try_get(column)),
                    "DOUBLE" => handle_result::<f64>(row.try_get(column)),
                    "VARCHAR" | "CHAR" | "TEXT" => handle_result::<String>(row.try_get(column)),
                    "VARBINARY" | "BINARY" | "BLOB" => {
                        let value : Result<Option<Vec<u8>>, sqlx::Error> = row.try_get(column);
                        match value {
                            Ok(value) => match value {
                                Some(value) => format!("{:?}", value),
                                None => String::from("NULL")
                            } 
                            Err(_) => String::from("NULL")
                        }
                    },
                    "DATETIME" => handle_result::<NaiveDateTime>(row.try_get(column)),
                    "DATE" => handle_result::<NaiveDate>(row.try_get(column)),
                    "TIME" => handle_result::<NaiveTime>(row.try_get(column)),
                    "TIMESTAMP" => handle_result::<DateTime<Utc>>(row.try_get(column)),
                    "JSON" => handle_result::<serde_json::Value>(row.try_get(column)),
                    _ => "NULL".to_string()
                };
                row_values.insert(column.to_string(), value_str);
            }
            result.values.push(row_values);
        }

        Ok(result)
    }
}

