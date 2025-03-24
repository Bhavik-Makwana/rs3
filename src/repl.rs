use crate::table::{self, Row};
use std::io;
enum StatementResult {
    Success,
    UnrecognizedStatement,
    PrepareSyntaxError(PrepareSyntaxError),
}

#[derive(Debug)]
enum PrepareSyntaxError {
    InvalidId,
    InvalidUsername,
    InvalidEmail,
    InvalidNumberOfArguments,
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
            return StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidNumberOfArguments);
        }

        match parts[1].parse::<u32>() {
            Ok(id) => statement.row_to_insert.id = id,
            Err(_) => return StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidId),
        }

        let username_bytes = parts[2].as_bytes();
        if username_bytes.len() > 32 {
            return StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidUsername);
        }
        statement.row_to_insert.username = [0; 32];
        statement.row_to_insert.username[..username_bytes.len()].copy_from_slice(username_bytes);

        let email_bytes = parts[3].as_bytes();
        if email_bytes.len() > 255 {
            return StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidEmail);
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
                    StatementResult::PrepareSyntaxError(error) => {
                        println!("Syntax error. Could not parse statement.");
                        println!("Error: {:?}", error);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_statement_insert() {
        let mut statement = Statement::new();
        let input = "insert 1 user1 user1@example.com";
        
        let result = prepare_statement(input, &mut statement);
        assert!(matches!(result, StatementResult::Success));
        assert!(matches!(statement.statement_type, StatementType::Insert));
        assert_eq!(statement.row_to_insert.id, 1);
        
        // Check username was properly copied
        let expected_username = "user1".as_bytes();
        assert_eq!(&statement.row_to_insert.username[..expected_username.len()], expected_username);
        
        // Check email was properly copied
        let expected_email = "user1@example.com".as_bytes();
        assert_eq!(&statement.row_to_insert.email[..expected_email.len()], expected_email);
    }

    #[test]
    fn test_prepare_statement_insert_syntax_error() {
        let mut statement = Statement::new();
        
        // Test too few arguments
        let result = prepare_statement("insert 1 user1", &mut statement);
        assert!(matches!(result, StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidNumberOfArguments)));

        // Test invalid ID
        let result = prepare_statement("insert abc user1 email@test.com", &mut statement);
        assert!(matches!(result, StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidId)));

        // Test username too long (> 32 bytes)
        let long_username = "a".repeat(33);
        let result = prepare_statement(
            &format!("insert 1 {} email@test.com", long_username),
            &mut statement
        );
        assert!(matches!(result, StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidUsername)));

        // Test email too long (> 255 bytes)
        let long_email = "a".repeat(256);
        let result = prepare_statement(
            &format!("insert 1 user1 {}", long_email),
            &mut statement
        );
        assert!(matches!(result, StatementResult::PrepareSyntaxError(PrepareSyntaxError::InvalidEmail)));
    }

    #[test]
    fn test_prepare_statement_select() {
        let mut statement = Statement::new();
        let result = prepare_statement("select", &mut statement);
        
        assert!(matches!(result, StatementResult::Success));
        assert!(matches!(statement.statement_type, StatementType::Select));
    }

    #[test]
    fn test_prepare_statement_unrecognized() {
        let mut statement = Statement::new();
        let result = prepare_statement("invalid statement", &mut statement);
        
        assert!(matches!(result, StatementResult::UnrecognizedStatement));
    }

    #[test]
    fn test_execute_meta_command() {
        let result = execute_meta_command(".unknown");
        assert!(matches!(result, MetaCommandResult::UnrecognizedCommand));
        
        // Note: We can't easily test ".exit" as it calls std::process::exit()
    }
}
