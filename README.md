# Run
Debug print an AST: `cd parser && cargo run <file.pax>` - test files available in `parser/test_files`

# Tests
`cd parser && cargo test` - runs all test files and verifies no errors occured. no unit tests/fuzz tests yet.

# Example Usage

```rust
fn main() -> Result<(), Box<dyn Error>> {
    // read a files source
    let file_name = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("test_file.pax".to_owned());
    let source = std::fs::read_to_string(&file_name).unwrap();

    // parse it into a pax AST
    let ast = Parser::new(&source).pax();

    // print results
    println!("-------DONE-------");
    match ast {
        Ok(ast) => println!("parsed AST: {:#?}", ast),
        Err(e) => e.print_with_file(&file_name, &source)?,
    };
    Ok(())
}

```
