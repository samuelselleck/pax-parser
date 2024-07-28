use crate::ast::PaxAst;
use crate::lexer::{Span, Token, TokenIterator, TokenKind};
use crate::utils::MultiPeek;

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
    tokens: MultiPeek<TokenIterator<'src>>,
    context_stack: Vec<&'static str>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            tokens: MultiPeek::new(TokenIterator::new(source)),
            context_stack: Vec::new(),
        }
    }

    pub fn pax(&mut self) -> Result<PaxAst, PaxParseError> {
        let mut templates = vec![];
        let mut settings = vec![];
        loop {
            match self.peek_token() {
                TokenKind::OpenAngBrack
                | TokenKind::For
                | TokenKind::If
                | TokenKind::Slot
                | TokenKind::Comment => templates.extend(self.template()?),
                TokenKind::AtSymbol => settings.extend(self.settings()?),
                TokenKind::EOF => break,
                _ => {
                    return Err(self.error([
                        TokenKind::OpenAngBrack,
                        TokenKind::AtSymbol,
                        TokenKind::EOF,
                    ]))
                }
            };
        }
        Ok(PaxAst {
            templates,
            settings,
        })
    }

    pub fn is_at_eof(&mut self) -> bool {
        self.peek_token() == TokenKind::EOF
    }

    fn peek_token(&mut self) -> TokenKind {
        self.tokens.peek().map_or(TokenKind::EOF, |t| t.kind)
    }

    fn peek_nth_token(&mut self, i: usize) -> TokenKind {
        self.tokens.peek_nth(i).map_or(TokenKind::EOF, |t| t.kind)
    }

    fn next_token_if(&mut self, f: impl FnOnce(TokenKind) -> bool) -> Option<Token> {
        self.tokens.next_if(|t| f(t.kind))
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().unwrap_or_else(|| {
            let len = self.tokens.inner().src.len();
            Token {
                span: Span {
                    start: len - 1,
                    end: len,
                },
                kind: TokenKind::EOF,
            }
        })
    }

    fn push_context(&mut self, context: &'static str) {
        self.context_stack.push(context);
        // println!(
        //     "{}entered: {:?}",
        //     " ".repeat(self.context_stack.len()),
        //     self.context_stack.last()
        // );
    }

    fn pop_context(&mut self) {
        // println!(
        //     "{}exited:  {:?}",
        //     " ".repeat(self.context_stack.len()),
        //     self.context_stack.pop()
        // );
        self.context_stack.pop();
    }

    fn source_of(&self, span: Span) -> &str {
        &self.tokens.inner().src[span.as_range()]
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
