use std::fmt::Display;

use crate::connector::{MySqlResult, MySqlRowsAffected, MySqlTable};

const TOP_LEFT: &str = "┌";
const TOP_RIGHT: &str = "┐";
const BOTTOM_LEFT: &str = "└";
const BOTTOM_RIGHT: &str = "┘";
const HORIZONTAL: &str = "─";
const VERTICAL: &str = "│";

const CROSS: &str = "┼";
const TOP_T: &str = "┬";
const BOTTOM_T: &str = "┴";
const LEFT_T: &str = "├";
const RIGHT_T: &str = "┤";

// TODO write!

impl Display for MySqlTable {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_lengths = self.headers.iter().map(|s| s.len()).collect::<Vec<usize>>();

        for row in &self.values {
            for (i, column_name) in self.headers.iter().enumerate() {
                let column = row.get(column_name).unwrap();
                max_lengths[i] = max_lengths[i].max(column.len());
            }
        }

        // Top separator
        print!("{TOP_LEFT}");
        for i in 0..max_lengths.len() {
            for _ in 0..max_lengths[i] + 2 {
                print!("{HORIZONTAL}");
            }
            if i != max_lengths.len() - 1 {
                print!("{TOP_T}");
            }
        }
        println!("{TOP_RIGHT}");

        // Print headers
        print!("{VERTICAL} ");
        for (header, max_length) in self.headers.iter().zip(max_lengths.iter()) {
            print!("{:>width$} {VERTICAL} ", header, width = max_length);
        }
        println!();

        // Header separator
        print!("{LEFT_T}");
        for i in 0..max_lengths.len() {
            for _ in 0..max_lengths[i] + 2 {
                print!("{HORIZONTAL}");
            }
            if i != max_lengths.len() - 1 {
                print!("{CROSS}");
            }
        }
        println!("{RIGHT_T}");

        // Print rows
        for row in &self.values {
            print!("{VERTICAL} ");
            for (i, column_name) in self.headers.iter().enumerate() {
                let column = row.get(column_name).unwrap();
                print!("{:>width$} {VERTICAL} ", column, width = max_lengths[i]);
            }
            println!();
        }

        // Bottom separator
        print!("{BOTTOM_LEFT}");
        for i in 0..max_lengths.len() {
            for _ in 0..max_lengths[i] + 2 {
                print!("{HORIZONTAL}");
            }
            if i != max_lengths.len() - 1 {
                print!("{BOTTOM_T}");
            }
        }
        println!("{BOTTOM_RIGHT}");
        println!();

        Ok(())
    }
}

impl Display for MySqlRowsAffected {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.affected_rows == 0 {
            println!("No rows affected");
            return Ok(());
        } else if self.affected_rows == 1 {
            println!("1 Row affected");
            return Ok(());
        }
        print!("{} Rows affected", self.affected_rows);
        Ok(())
    }
}

impl Display for MySqlResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MySqlResult::Table(table) => write!(f, "{}", table),
            MySqlResult::RowsAffected(rows_affected) => write!(f, "{}", rows_affected),
        }
    }
}
