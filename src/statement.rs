use crate::{
    layout::{ROW_SIZE, TABLE_MAX_ROWS},
    row::{Row, RowBytes},
    table::Table,
};
use anyhow::Result;
use thiserror::Error;

type FilteredRows = Vec<Row>;

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

    let bytes: RowBytes = row.into();
    let (page_num, offset) = table.row_slot(table.row_count);
    let page = table.pager.page(page_num)?;
    // let page = table.pager.pages[page_num].as_mut().unwrap();
    page[offset..offset + ROW_SIZE].copy_from_slice(&bytes);
    table.row_count += 1;
    Ok(())
}

pub fn execute_select(table: &mut Table) -> Result<FilteredRows> {
    let mut rows: FilteredRows = vec![];
    for i in 0..table.row_count {
        let (page_num, offset) = table.row_slot(i);
        let page = table.pager.page(page_num)?;
        let row: RowBytes = page[offset..offset + ROW_SIZE].try_into()?;
        rows.push(row.into());
    }

    Ok(rows)
}
