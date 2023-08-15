use crate::{
    cursor::Cursor,
    layout::{ROW_SIZE, TABLE_MAX_ROWS},
    row::{Row, RowBytes},
    table::Table,
};
use anyhow::Result;
use thiserror::Error;

#[derive(Debug)]
pub enum StatementType {
    Insert(Box<Row>),
    Select,
}

#[derive(Debug)]
pub struct Statement {
    pub kind: StatementType,
}

impl Statement {
    fn insert(row: Row) -> Self {
        Self {
            kind: StatementType::Insert(Box::new(row)),
        }
    }

    fn select() -> Self {
        Self {
            kind: StatementType::Select,
        }
    }
}

#[derive(Error, Debug)]
enum PrepareError {
    #[error("Unrecognized command: '{0}'")]
    Unrecognized(String),
    #[error("Invalid syntax")]
    InvalidSyntax,
}

#[derive(Error, Debug)]
enum ExecutionError {
    #[error("Table is full")]
    TableFull,
}

pub fn prepare_statement(input: String) -> Result<Statement> {
    let mut parts = input.split_whitespace();
    match parts.next() {
        Some("insert") => {
            let args: Vec<&str> = parts.take(3).collect();
            if args.len() != 3 {
                return Err(PrepareError::InvalidSyntax.into());
            }
            let id: u16 = args[0].parse()?;
            Ok(Statement::insert(Row::new(id, args[1], args[2])))
        }
        Some("select") => Ok(Statement::select()),
        Some(keyword) => {
            let keyword = keyword.to_string();
            println!("Unrecognized keyword: '{}'", keyword);
            Err(PrepareError::Unrecognized(keyword).into())
        }
        None => panic!("Unreachable arm"),
    }
}

pub fn execute_insert(row: Row, table: &mut Table) -> Result<()> {
    if table.row_count >= TABLE_MAX_ROWS {
        return Err(ExecutionError::TableFull.into());
    }

    let cursor = Cursor::at_table_end(table);
    let bytes: RowBytes = row.into();

    let (page_num, offset) = cursor.position();
    let page = table.pager.page(page_num)?;
    page[offset..offset + ROW_SIZE].copy_from_slice(&bytes);
    table.row_count += 1;
    Ok(())
}

pub fn execute_select(table: &mut Table) -> Result<Vec<Row>> {
    let mut rows: Vec<Row> = vec![];
    let mut cursor = Cursor::at_table_start(table);
    while !cursor.end_of_table {
        let (page_num, offset) = cursor.position();
        let page = cursor.table.pager.page(page_num)?;
        let row: RowBytes = page[offset..offset + ROW_SIZE].try_into()?;
        rows.push(row.into());
        cursor.advance();
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{pager::Pager, row::Row, table::Table};
    use tempfile::NamedTempFile;

    #[test]
    fn execute_select_with_data_succeeds() {
        let file = NamedTempFile::new_in("target").unwrap();
        let pager = Pager::new(file.path()).unwrap();
        let mut table = Table::new(pager);
        for _ in 0..5 {
            execute_insert(Row::default(), &mut table).unwrap();
        }
        let rows = execute_select(&mut table).unwrap();
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn execute_insert_exceeding_row_limit_fails() {
        let file = NamedTempFile::new_in("target").unwrap();
        let pager = Pager::new(file.path()).unwrap();
        let mut table = Table::new(pager);
        for _ in 0..TABLE_MAX_ROWS {
            let row = Row::new(u16::MAX, "a".repeat(32).as_str(), "a".repeat(255).as_str());
            execute_insert(row, &mut table).unwrap();
        }
        let result = execute_insert(Row::default(), &mut table);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn execute_insert_input_exceeding_length_fails() {
        let file = NamedTempFile::new_in("target").unwrap();
        let pager = Pager::new(file.path()).unwrap();
        let mut table = Table::new(pager);
        let row = Row::new(u16::MAX, "a".repeat(33).as_str(), "a".repeat(256).as_str());
        let result = execute_insert(row, &mut table);
        assert!(result.is_err());
    }
}
