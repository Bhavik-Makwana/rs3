use crate::table::{self, Row};
use std::io;
enum StatementResult {
    Success,
    UnrecognizedStatement,
    PrepareSyntaxError,
}

enum MetaCommandResult {
    _Success,
    UnrecognizedCommand,
}

struct Statement {
    statement_type: StatementType,
    statement: String,
    row_to_insert: Row,
}

impl Statement {
    fn new() -> Self {
        Statement {
            statement_type: StatementType::Insert,
            statement: String::new(),
            row_to_insert: Row::new(),
        }
    }
}

enum StatementType {
    Insert,
    Select,
}

enum ExecuteResult {
    Success,
    TableFull,
}

fn execute_statement(statement: Statement, table: &mut table::Table) -> ExecuteResult {
    match statement.statement_type {
        StatementType::Insert => execute_insert(statement, table),
        StatementType::Select => execute_select(statement, table),
    }
}

fn execute_insert(statement: Statement, table: &mut table::Table) -> ExecuteResult {
    if table.num_rows >= table::TABLE_MAX_ROWS {
        return ExecuteResult::TableFull;
    }
    let row_to_insert = statement.row_to_insert;
    let (page, idx) = table.row_slot(table.num_rows);
    table.serialize_row(&row_to_insert, (page, idx));
    table.num_rows += 1;
    ExecuteResult::Success
}

fn execute_select(_statement: Statement, table: &mut table::Table) -> ExecuteResult {
    let mut row;
    for i in 0..table.num_rows {
        let (page, idx) = table.row_slot(i);
        row = table.deserialize_row(page, idx);
        println!("{}", row);
    }
    ExecuteResult::Success
}

fn execute_meta_command(cmd: &str) -> MetaCommandResult {
    if cmd == ".exit" {
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

fn prepare_statement(input: &str, statement: &mut Statement) -> StatementResult {
    if input.starts_with("insert") {
        statement.statement_type = StatementType::Insert;
        statement.statement = input.to_string();
        let parts = input.split_whitespace().collect::<Vec<&str>>();
        println!("parts {:?}", parts);

        if parts.len() != 4 {
            return StatementResult::PrepareSyntaxError;
        }

        match parts[1].parse::<u32>() {
            Ok(id) => statement.row_to_insert.id = id,
            Err(_) => return StatementResult::PrepareSyntaxError,
        }

        let username_bytes = parts[2].as_bytes();
        if username_bytes.len() > 32 {
            return StatementResult::PrepareSyntaxError;
        }
        statement.row_to_insert.username = [0; 32];
        statement.row_to_insert.username[..username_bytes.len()].copy_from_slice(username_bytes);

        let email_bytes = parts[3].as_bytes();
        if email_bytes.len() > 255 {
            return StatementResult::PrepareSyntaxError;
        }
        statement.row_to_insert.email = [0; 255];
        statement.row_to_insert.email[..email_bytes.len()].copy_from_slice(email_bytes);

        StatementResult::Success
    } else if input == "select" {
        statement.statement_type = StatementType::Select;
        statement.statement = input.to_string();
        StatementResult::Success
    } else {
        StatementResult::UnrecognizedStatement
    }
}

struct InputBuffer {
    buffer: String,
    buffer_length: usize,
    input_length: usize,
}

impl InputBuffer {
    fn new() -> Self {
        InputBuffer {
            buffer: String::new(),
            buffer_length: 0,
            input_length: 0,
        }
    }

    fn read_input(&mut self) {
        let mut temp = String::new();
        match io::stdin().read_line(&mut temp) {
            Ok(bytes_read) if bytes_read > 0 => {
                self.buffer = temp.trim_end().to_string();
                self.buffer_length = self.buffer.len();
                self.input_length = self.buffer_length;
            }
            _ => {
                println!("Error reading input");
                std::process::exit(1);
            }
        }
    }
}

fn print_prompt() {
    print!("db > ");
}

pub fn run() {
    let mut table = table::Table::new();
    println!("Hello, world!");
    let mut input_buffer = InputBuffer::new();
    loop {
        print_prompt();
        input_buffer.read_input();
        match input_buffer.buffer.trim() {
            cmd if cmd.starts_with(".") => match execute_meta_command(cmd) {
                MetaCommandResult::_Success => continue,
                MetaCommandResult::UnrecognizedCommand => {
                    println!("Unrecognized command '{}'.", cmd);
                    continue;
                }
            },
            input => {
                let mut statement = Statement::new();
                match prepare_statement(input, &mut statement) {
                    StatementResult::Success => {
                        println!("execute_statement");
                        match execute_statement(statement, &mut table) {
                            ExecuteResult::Success => {
                                println!("Executed.");
                            }
                            ExecuteResult::TableFull => {
                                println!("Error: Table full.");
                            }
                        }
                    }
                    StatementResult::PrepareSyntaxError => {
                        println!("Syntax error. Could not parse statement.");
                    }
                    StatementResult::UnrecognizedStatement => {
                        println!(
                            "Unrecognized keyword at start of '{}'.",
                            statement.statement
                        );
                    }
                }
            }
        }
    }
}
