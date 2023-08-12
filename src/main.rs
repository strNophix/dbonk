use anyhow::Result;
use dbonk::{pager::Pager, statement::*, table::Table};
use std::{
    env,
    io::{stdin, stdout, Write},
    process,
};

fn handle_meta_command(input: String, table: &mut Table) {
    let mut parts = input.split_whitespace();
    let command = parts.next().expect("command not found");
    match command {
        ".exit" => {
            table.close();
            process::exit(0);
        }
        _ => println!("Unrecognized command '{}'", input),
    }
}

fn execute_statement(statement: Statement, table: &mut Table) -> Result<()> {
    match statement.kind {
        StatementType::Insert(row) => execute_insert(*row, table),
        StatementType::Select => {
            let rows = execute_select(table)?;
            for row in rows {
                println!("{:?}", row);
            }
            Ok(())
        }
    }
}

fn handle_statement(input: String, table: &mut Table) -> Result<()> {
    match prepare_statement(input) {
        Ok(statement) => execute_statement(statement, table),
        Err(_) => Ok(()),
    }
}

fn read_input(input: &mut String) {
    print!("sqlite> ");
    stdout().flush().unwrap();
    input.clear();
    stdin().read_line(input).unwrap();
    input.pop(); // Pop the newline character.
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Must supply a database filename.");
        process::exit(1);
    }

    let pager = Pager::new(&args[1])?;
    let mut table = Table::new(pager);

    let mut input = String::new();
    loop {
        read_input(&mut input);

        if input.is_empty() {
            continue;
        }

        match input.starts_with('.') {
            true => handle_meta_command(input.clone(), &mut table),
            false => handle_statement(input.clone(), &mut table)?,
        }
    }
}
