use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::{Comment, Field, FieldOrComment, Identifier},
        expression::Object,
    },
    lexer::Token,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    // this function is used to peek forward far enough to be sure
    // that the next ast node is a map. This is needed to dissambiguate
    // from an expression wrapped in { .. } and the start of a for/if loop block.
    // this function is by far the place with the largest amount of lookahead
    pub fn is_map_next(&mut self) -> bool {
        // must be an identifier or open curly bracket first
        let mut lookahead = 0;
        // skip a identifier if present (name of map, such as LinearGradient {})
        if self.tokens.peek() == Token::Identifier {
            lookahead += 1;
        }
        // expect a open curly bracket
        if self.tokens.peek_nth(lookahead) != Token::OpenCurlBrack {
            return false;
        }
        lookahead += 1;
        // skip an unknown ammount of legal comment blocks.
        // this isn't great since it's technically an arbitrary ammount
        // of lookahead, but  should very very seldom be more than 3 or 4 or so
        while self.tokens.peek_nth(lookahead) == Token::Comment {
            lookahead += 1;
        }
        // expect an identifier
        if self.tokens.peek_nth(lookahead) != Token::Identifier {
            return false;
        }
        lookahead += 1;
        // then a colon
        if self.tokens.peek_nth(lookahead) != Token::Colon {
            return false;
        }
        // now we are sure, this must be a map
        return true;
    }

    #[token_context("Object (<optional ident> {foo: .. bar: ..})")]
    pub fn object(&mut self) -> Result<Object, PaxParseError> {
        let name = self
            .tokens
            .next_if(|t| t == Token::Identifier)
            .map(|t| Identifier(t.span));
        let fields = self.map()?;
        Ok(Object { name, fields })
    }

    #[token_context("Map ({foo: .. bar: ..})")]
    pub fn map(&mut self) -> Result<Vec<FieldOrComment>, PaxParseError> {
        self.expect(Token::OpenCurlBrack)?;
        let mut entries = vec![];
        loop {
            entries.push(match self.tokens.peek() {
                Token::Identifier => {
                    let key = self.tokens.next();
                    self.expect(Token::Colon)?;
                    let value = self.literal_or_wrapped_expression()?;
                    //skip comma between fields
                    self.tokens.next_if(|t| t == Token::Comma);
                    FieldOrComment::Field(Field {
                        key: Identifier(key.span),
                        value,
                    })
                }
                Token::Comment => FieldOrComment::Comment(Comment(self.tokens.next().span)),
                Token::CloseCurlBrack => {
                    self.tokens.next();
                    break;
                }
                _ => return Err(self.error([Token::Identifier])),
            });
        }
        Ok(entries)
    }
}
