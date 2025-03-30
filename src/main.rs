use rsqlite3::repl;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let table_name = if args.len() > 1 {
        &args[1]
    } else {
        "test.db" // Default table name if none provided
    };

    repl::run(table_name);
}
