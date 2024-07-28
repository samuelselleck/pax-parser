## Run
Debug print an AST: `cd parser && cargo run <file.pax>` - test files are available in `parser/test_files`

## Tests
`cd parser && cargo test` - runs all test files and verifies no errors occurred. No unit tests/fuzz tests yet.

## Example Usage
```rust
use pax_parser::Parser;

fn main() {
    let source = std::fs::read_to_string("your_file.pax").unwrap();
    let ast = Parser::new(&source).pax();
    
    match ast {
        Ok(ast) => println!("Parsed AST: {:#?}", ast),
        Err(e) => e.print_with_file("your_file.pax", &source).unwrap(),
    }
}
```

## Project Structure
The project is organized into the following modules:
- `lexer`: Handles tokenization of the input source
- `parser`: Contains the main parsing logic
- `ast`: Defines the structure of the Abstract Syntax Tree
- `utils`: Provides utility functions and structures (e.g., MultiPeek iterator)
