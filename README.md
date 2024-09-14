## About
a parser for the pax user interface DSL:
```html
<Frame width=20%>
	<Text text={self.message} class=centered class=small id=text @click=self.increment/>
	<Rectangle class=centered class=small 
	     fill={hsl(ticks, 75%, 50%)}
	    corner_radii={radii(10.0,10.0,10.0,10.0)}
	/>
</Frame>
if self.activated {
	<Stacker cells=5>
	for i in 0..5 {
		<Stacker cells=5 direction=Vertical>
		for j in 0..5 {
			<Frame>
				<Text text="_R" class=centered width=80% height=80% id=text />
				<Rectangle class=centered width=50% height=50% @click=self.increment
				     fill={hsl(ticks + 20*i, (50 + j*10)%, 50%)}
				    corner_radii={radii(10.0,10.0,10.0,10.0)}
				/>
			</Frame>
		}
		</Stacker>
	}
	</Stacker>
}

@settings {
     @mount: handle_mount,
     @pre_render: handle_pre_render,
     .centered {
        x: 50%
        y: 50%
        anchor_x: 50%
        anchor_y: 50%
    }
    .small {
        width: 120px
        height: 120px
    }
    #text {
        style: {
                font: {system("Times New Roman", Normal, Bold)},
                font_size: 32px,
                fill: BLACK,
                align_vertical: Center,
                align_horizontal: Center,
                align_multiline: Center
        }
    }
}
```
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

## Run
Debug print an AST: `cd parser && cargo run <file.pax>` - test files are available in `parser/test_files`

## Tests
`cd parser && cargo test` - runs all test files and verifies no errors occurred. No unit tests/fuzz tests yet.

## Project Structure
The project is organized into these modules:
- `lexer`: Handles tokenization of the input source
- `parser`: Contains the main parsing logic
- `ast`: Defines the structure of the Abstract Syntax Tree
- `utils`: Provides utility functions and structures (e.g., MultiPeek iterator)
