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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_lengths = self.headers.iter().map(|s| s.len()).collect::<Vec<usize>>();

        for row in &self.values {
            for (i, column_name) in self.headers.iter().enumerate() {
                let column = row.get(column_name).unwrap();
                max_lengths[i] = max_lengths[i].max(column.len());
            }
        }

        // Top separator
        write!(f, "{TOP_LEFT}")?;
        for i in 0..max_lengths.len() {
            for _ in 0..max_lengths[i] + 2 {
                write!(f, "{HORIZONTAL}")?;
            }
            if i != max_lengths.len() - 1 {
                write!(f, "{TOP_T}")?;
            }
        }
        writeln!(f, "{TOP_RIGHT}")?;

        // Print headers
        write!(f, "{VERTICAL} ")?;
        for (header, max_length) in self.headers.iter().zip(max_lengths.iter()) {
            write!(f, "{:>width$} {VERTICAL} ", header, width = max_length)?;
        }
        writeln!(f)?;

        // Header separator
        write!(f, "{LEFT_T}")?;
        for i in 0..max_lengths.len() {
            for _ in 0..max_lengths[i] + 2 {
                write!(f, "{HORIZONTAL}")?;
            }
            if i != max_lengths.len() - 1 {
                write!(f, "{CROSS}")?;
            }
        }
        writeln!(f, "{RIGHT_T}")?;

        // Print rows
        for row in &self.values {
            write!(f, "{VERTICAL} ")?;
            for (i, column_name) in self.headers.iter().enumerate() {
                let column = row.get(column_name).unwrap();
                write!(f, "{:>width$} {VERTICAL} ", column, width = max_lengths[i])?;
            }
            writeln!(f)?;
        }

        // Bottom separator
        write!(f, "{BOTTOM_LEFT}")?;
        for i in 0..max_lengths.len() {
            for _ in 0..max_lengths[i] + 2 {
                write!(f, "{HORIZONTAL}")?;
            }
            if i != max_lengths.len() - 1 {
                write!(f, "{BOTTOM_T}")?;
            }
        }
        writeln!(f, "{BOTTOM_RIGHT}")?;

        Ok(())
    }
}

impl Display for MySqlRowsAffected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.affected_rows == 0 {
            writeln!(f, "No rows affected")
        } else if self.affected_rows == 1 {
            writeln!(f, "1 Row affected")
        } else {
            writeln!(f, "{} Rows affected", self.affected_rows)
        }
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
