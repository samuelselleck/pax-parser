use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::{Comment, Handler, Identifier},
        settings::{Class, Id, SettingsEntry},
    },
    lexer::TokenKind,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Settings")]
    pub fn settings(&mut self) -> Result<Vec<SettingsEntry>, PaxParseError> {
        let [_, ident, _] = self.expect_sequence([
            TokenKind::AtSymbol,
            TokenKind::Identifier,
            TokenKind::OpenCurlBrack,
        ])?;
        if self.source_of(ident.span) != "settings" {
            return Err(PaxParseError::new("expected settings block")
                .annotation(ident.span, "only settings allowed in top level context"));
        }
        let mut entries = vec![];
        loop {
            entries.push(match self.peek_token() {
                TokenKind::AtSymbol => SettingsEntry::Handler(self.handler()?),
                TokenKind::Period => SettingsEntry::Class(self.class()?),
                TokenKind::Hashtag => SettingsEntry::Id(self.id()?),
                TokenKind::Comment => {
                    SettingsEntry::Comment(Comment(self.expect(TokenKind::Comment)?.span))
                }
                TokenKind::CloseCurlBrack => {
                    self.tokens.next();
                    break;
                }
                _ => {
                    return Err(self.error([
                        TokenKind::AtSymbol,
                        TokenKind::Period,
                        TokenKind::Hashtag,
                        TokenKind::Comment,
                        TokenKind::CloseCurlBrack,
                    ]));
                }
            });
            //skip commas if they exist
            self.next_token_if(|t| t == TokenKind::Comma);
        }
        Ok(entries)
    }

    #[token_context("Handler (@handler=foo)")]
    fn handler(&mut self) -> Result<Handler, PaxParseError> {
        let [_, name, _, ident] = self.expect_sequence([
            TokenKind::AtSymbol,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Identifier,
        ])?;
        Ok(Handler {
            key: Identifier(name.span),
            value: Identifier(ident.span),
        })
    }

    #[token_context("Class (.a_class {..})")]
    fn class(&mut self) -> Result<Class, PaxParseError> {
        self.expect(TokenKind::Period)?;
        let name = self.expect(TokenKind::Identifier)?;
        let body = self.map()?;
        Ok(Class {
            name: Identifier(name.span),
            binding: body,
        })
    }

    #[token_context("Id (#a_class {..})")]
    fn id(&mut self) -> Result<Id, PaxParseError> {
        self.expect(TokenKind::Hashtag)?;
        let name = self.expect(TokenKind::Identifier)?;
        let body = self.map()?;
        Ok(Id {
            name: Identifier(name.span),
            binding: body,
        })
    }
}
