use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::{Comment, Handler, Identifier},
        settings::{Class, Id, SettingsEntry},
    },
    lexer::Token,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Settings")]
    pub fn settings(&mut self) -> Result<Vec<SettingsEntry>, PaxParseError> {
        let [_, ident, _] =
            self.expect_sequence([Token::AtSymbol, Token::Identifier, Token::OpenCurlBrack])?;
        if self.source_of(ident.span) != "settings" {
            return Err(PaxParseError::new("expected settings block")
                .annotation(ident.span, "only settings allowed in top level context"));
        }
        let mut entries = vec![];
        loop {
            entries.push(match self.tokens.peek() {
                Token::AtSymbol => SettingsEntry::Handler(self.handler()?),
                Token::Period => SettingsEntry::Class(self.class()?),
                Token::Hashtag => SettingsEntry::Id(self.id()?),
                Token::Comment => {
                    SettingsEntry::Comment(Comment(self.expect(Token::Comment)?.span))
                }
                Token::CloseCurlBrack => {
                    self.tokens.next();
                    break;
                }
                _ => {
                    return Err(self.error([
                        Token::AtSymbol,
                        Token::Period,
                        Token::Hashtag,
                        Token::Comment,
                        Token::CloseCurlBrack,
                    ]));
                }
            });
            //skip commas if they exist
            self.tokens.next_if(|t| t == Token::Comma);
        }
        Ok(entries)
    }

    #[token_context("Handler (@handler=foo)")]
    fn handler(&mut self) -> Result<Handler, PaxParseError> {
        let [_, name, _, ident] = self.expect_sequence([
            Token::AtSymbol,
            Token::Identifier,
            Token::Colon,
            Token::Identifier,
        ])?;
        Ok(Handler {
            key: Identifier(name.span),
            value: Identifier(ident.span),
        })
    }

    #[token_context("Class (.a_class {..})")]
    fn class(&mut self) -> Result<Class, PaxParseError> {
        self.expect(Token::Period)?;
        let name = self.expect(Token::Identifier)?;
        let body = self.map()?;
        Ok(Class {
            name: Identifier(name.span),
            binding: body,
        })
    }

    #[token_context("Id (#a_class {..})")]
    fn id(&mut self) -> Result<Id, PaxParseError> {
        self.expect(Token::Hashtag)?;
        let name = self.expect(Token::Identifier)?;
        let body = self.map()?;
        Ok(Id {
            name: Identifier(name.span),
            binding: body,
        })
    }
}
