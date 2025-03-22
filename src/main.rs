use std::io;
fn main() {
    println!("Hello, world!");
    let mut input_buffer = InputBuffer::new();
    loop {
        print_prompt();
        input_buffer.read_input();
        match input_buffer.buffer.trim() {
            ".exit" => std::process::exit(0),
            cmd => println!("Unrecognized command '{}'.", cmd),
        }
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
