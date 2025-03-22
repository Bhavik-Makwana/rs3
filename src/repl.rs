use std::io;

enum StatementResult {
    Success,
    UnrecognizedStatement,
}

enum MetaCommandResult {
    Success,
    UnrecognizedCommand,
}

struct Statement {
    statement_type: StatementType,
    statement: String,
}

impl Statement {
    fn new() -> Self {
        Statement {
            statement_type: StatementType::Insert,
            statement: String::new(),
        }
    }
}

enum StatementType {
    Insert,
    Select,
}

fn execute_statement(statement: Statement) {
    match statement.statement_type {
        StatementType::Insert => println!("Executing insert"),
        StatementType::Select => println!("Executing select"),
    }
}

fn execute_meta_command(cmd: &str) -> MetaCommandResult {
    if cmd == ".exit" {
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

fn prepare_statement(input: &str, statement: &mut Statement) -> StatementResult {
    if input == "insert" {
        statement.statement_type = StatementType::Insert;
        statement.statement = input.to_string();
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
    println!("Hello, world!");
    let mut input_buffer = InputBuffer::new();
    loop {
        print_prompt();
        input_buffer.read_input();
        match input_buffer.buffer.trim() {
            cmd if cmd.starts_with(".") => match execute_meta_command(cmd) {
                MetaCommandResult::Success => continue,
                MetaCommandResult::UnrecognizedCommand => {
                    println!("Unrecognized command '{}'.", cmd);
                    continue;
                }
            },
            input => {
                let mut statement = Statement::new();
                match prepare_statement(input, &mut statement) {
                    StatementResult::Success => {
                        execute_statement(statement);
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