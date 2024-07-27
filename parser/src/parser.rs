use crate::ast::PaxAst;
use crate::lexer::{Span, Token, TokenIterator};

use self::errors::PaxParseError;

pub mod common;
mod errors;
pub mod expression;
pub mod literal;
pub mod settings;
pub mod template;
pub mod value;

/// Parses a pax source file into an AST.
pub struct Parser<'src> {
    tokens: TokenIterator<'src>,
    context_stack: Vec<&'static str>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            tokens: TokenIterator::new(source),
            context_stack: Vec::new(),
        }
    }

    pub fn pax(&mut self) -> Result<PaxAst, PaxParseError> {
        let mut templates = vec![];
        let mut settings = vec![];
        loop {
            match self.tokens.peek() {
                Token::OpenAngBrack | Token::For | Token::If | Token::Slot | Token::Comment => {
                    templates.extend(self.template()?)
                }
                Token::AtSymbol => settings.extend(self.settings()?),
                Token::EOF => break,
                _ => return Err(self.error([Token::OpenAngBrack, Token::AtSymbol, Token::EOF])),
            };
        }
        Ok(PaxAst {
            templates,
            settings,
        })
    }

    pub fn is_at_eof(&mut self) -> bool {
        self.tokens.peek() == Token::EOF
    }

    fn push_context(&mut self, context: &'static str) {
        self.context_stack.push(context);
    }

    fn pop_context(&mut self) {
        self.context_stack.pop();
    }

    fn source_of(&self, span: Span) -> &str {
        &self.tokens.src[span.as_range()]
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    /// Large test that runs all examples
    /// in the pax_files directory
    #[test]
    fn test_examples() {
        let pax_files = std::fs::read_dir("./pax_files").unwrap();
        let mut results = vec![];
        for dir_entr in pax_files {
            let path = dir_entr.unwrap().path();
            let contents = std::fs::read_to_string(&path).unwrap();
            let ast = Parser::new(&contents).pax();
            if let Err(e) = &ast {
                eprintln!("[{:?}] couldn't parse: {:?}", path, e);
            }
            if let Err(err) = ast {
                results.push((path, format!("{:?}", err)));
            }
        }
        if results.is_empty() {
            return;
        }

        for (name, res) in results {
            println!("{:?}: {}", name, res);
        }
        panic!("one or more pax example files failed to parse, run test with -- --nocapture")
    }
}
