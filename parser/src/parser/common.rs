use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::{Comment, Field, FieldOrComment, Identifier},
        expression::Object,
    },
    lexer::TokenKind,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    // this function is used to peek forward far enough to be sure
    // that the next ast node is a map. This is needed to dissambiguate
    // from an expression wrapped in { .. } and the start of a for/if loop block.
    // this function is by far the place with the largest amount of lookahead
    pub fn is_map_next(&mut self) -> bool {
        let mut lookahead = 0;
        // skip a identifier if present (name of map, such as LinearGradient {})
        if self.peek_token() == TokenKind::Identifier {
            lookahead += 1;
        }
        // expect a open curly bracket
        if self.peek_nth_token(lookahead) != TokenKind::OpenCurlBrack {
            return false;
        }
        lookahead += 1;
        // skip an unknown ammount of legal comment blocks.
        // this isn't great since it's technically an arbitrary ammount
        // of lookahead, but  should very very seldom be more than 3 or 4 or so
        while self.peek_nth_token(lookahead) == TokenKind::Comment {
            lookahead += 1;
        }
        // expect an identifier
        if self.peek_nth_token(lookahead) != TokenKind::Identifier {
            return false;
        }
        lookahead += 1;
        // then a colon
        if self.peek_nth_token(lookahead) != TokenKind::Colon {
            return false;
        }
        // now we are sure, this must be a map
        return true;
    }

    #[token_context("Object (<optional ident> {foo: .. bar: ..})")]
    pub fn object(&mut self) -> Result<Object, PaxParseError> {
        let name = self
            .next_token_if(|t| t == TokenKind::Identifier)
            .map(|t| Identifier(t.span));
        let fields = self.map()?;
        Ok(Object { name, fields })
    }

    #[token_context("Map ({foo: .. bar: ..})")]
    pub fn map(&mut self) -> Result<Vec<FieldOrComment>, PaxParseError> {
        self.expect(TokenKind::OpenCurlBrack)?;
        let mut entries = vec![];
        loop {
            entries.push(match self.peek_token() {
                TokenKind::Identifier => {
                    let key = self.next_token();
                    self.expect(TokenKind::Colon)?;
                    let value = self.literal_or_wrapped_expression()?;
                    //skip comma between fields
                    self.next_token_if(|t| t == TokenKind::Comma);
                    FieldOrComment::Field(Field {
                        key: Identifier(key.span),
                        value,
                    })
                }
                TokenKind::Comment => FieldOrComment::Comment(Comment(self.next_token().span)),
                TokenKind::CloseCurlBrack => {
                    self.tokens.next();
                    break;
                }
                _ => return Err(self.error([TokenKind::Identifier])),
            });
        }
        Ok(entries)
    }
}
