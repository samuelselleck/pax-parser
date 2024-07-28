use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::{Comment, Field, Handler, Identifier},
        expression::Expression,
        template::{Attribute, Binding, Conditional, Loop, MatchPattern, Tag, TemplateEntry},
    },
    lexer::TokenKind,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Template")]
    pub fn template(&mut self) -> Result<Vec<TemplateEntry>, PaxParseError> {
        let mut template: Vec<TemplateEntry> = vec![];
        loop {
            let entry = match self.peek_token() {
                TokenKind::CloseCurlBrack | TokenKind::AtSymbol | TokenKind::EOF => break,
                TokenKind::OpenAngBrack => {
                    if self.peek_nth_token(1) == TokenKind::Slash {
                        break;
                    } else {
                        TemplateEntry::Tag(self.tag()?)
                    }
                }
                TokenKind::For => TemplateEntry::Loop(self.for_loop()?),
                TokenKind::If => TemplateEntry::Conditional(self.condition()?),
                TokenKind::Slot => TemplateEntry::Slot(self.slot()?),
                TokenKind::Comment => {
                    TemplateEntry::Comment(Comment(self.expect(TokenKind::Comment)?.span))
                }
                _ => {
                    return Err(self.error([
                        TokenKind::CloseCurlBrack,
                        TokenKind::OpenAngBrack,
                        TokenKind::For,
                        TokenKind::If,
                        TokenKind::Slot,
                        TokenKind::Comment,
                    ]))
                }
            };
            template.push(entry);
        }
        Ok(template)
    }

    #[token_context("Tag pair (<tag>..</tag>)")]
    fn tag(&mut self) -> Result<Tag, PaxParseError> {
        self.expect(TokenKind::OpenAngBrack)?;
        let name = self.expect(TokenKind::Identifier)?;

        let mut attributes = vec![];
        loop {
            match self.peek_token() {
                TokenKind::CloseAngBrack | TokenKind::Slash => break,
                _ => {
                    attributes.push(self.attribute()?);
                }
            }
        }
        let body = match self.peek_token() {
            TokenKind::CloseAngBrack => {
                self.tokens.next();
                let template = self.template()?;
                let [_, _, ident, _] = self.expect_sequence([
                    TokenKind::OpenAngBrack,
                    TokenKind::Slash,
                    TokenKind::Identifier,
                    TokenKind::CloseAngBrack,
                ])?;
                if self.source_of(ident.span) != self.source_of(name.span) {
                    return Err(PaxParseError::new("unexpected closing tag")
                        .annotation(
                            ident.span,
                            format!(
                                "found closing tag with name {:?}",
                                self.source_of(ident.span)
                            ),
                        )
                        .annotation(name.span, "expected to close this next"));
                }
                template
            }
            TokenKind::Slash => {
                self.tokens.next();
                self.expect(TokenKind::CloseAngBrack)?;
                vec![]
            }
            _ => unreachable!("should have continued trying to parse attributes?"),
        };
        Ok(Tag {
            name: Identifier(name.span),
            attributes,
            body,
        })
    }

    #[token_context("Attribute (@handler=foo or key=value)")]
    fn attribute(&mut self) -> Result<Attribute, PaxParseError> {
        Ok(match self.peek_token() {
            TokenKind::AtSymbol => {
                let [_, key, _, value] = self.expect_sequence([
                    TokenKind::AtSymbol,
                    TokenKind::Identifier,
                    TokenKind::Assign,
                    TokenKind::Identifier,
                ])?;
                Attribute::Handler(Handler {
                    key: Identifier(key.span),
                    value: Identifier(value.span),
                })
            }
            TokenKind::Bind => {
                let [_, _, key, _, value] = self.expect_sequence([
                    TokenKind::Bind,
                    TokenKind::Colon,
                    TokenKind::Identifier,
                    TokenKind::Assign,
                    TokenKind::Identifier,
                ])?;
                Attribute::Binding(Binding {
                    key: Identifier(key.span),
                    value: Identifier(value.span),
                })
            }
            TokenKind::Identifier => {
                let [key, _] = self.expect_sequence([TokenKind::Identifier, TokenKind::Assign])?;
                Attribute::Field(Field {
                    key: Identifier(key.span),
                    value: self.literal_or_wrapped_expression()?,
                })
            }
            _ => {
                return Err(self.error([
                    TokenKind::AtSymbol,
                    TokenKind::Bind,
                    TokenKind::Identifier,
                ]))
            }
        })
    }

    #[token_context("For loop (for i in items {..})")]
    fn for_loop(&mut self) -> Result<Loop, PaxParseError> {
        self.expect(TokenKind::For)?;

        let pattern = if self.peek_token() == TokenKind::OpenParenth {
            let [_, t1, _, t2, _] = self.expect_sequence([
                TokenKind::OpenParenth,
                TokenKind::Identifier,
                TokenKind::Comma,
                TokenKind::Identifier,
                TokenKind::CloseParenth,
            ])?;
            MatchPattern::Tuple(Identifier(t1.span), Identifier(t2.span))
        } else {
            let ident = self.expect(TokenKind::Identifier)?;
            MatchPattern::Identifier(Identifier(ident.span))
        };

        self.expect(TokenKind::In)?;
        let source = self.expression()?;
        self.expect(TokenKind::OpenCurlBrack)?;
        let body = self.template()?;
        self.expect(TokenKind::CloseCurlBrack)?;
        Ok(Loop {
            pattern,
            source,
            body,
        })
    }

    #[token_context("Condition (if cond {..})")]
    fn condition(&mut self) -> Result<Conditional, PaxParseError> {
        self.expect(TokenKind::If)?;
        let condition = self.expression()?;
        self.expect(TokenKind::OpenCurlBrack)?;
        let body = self.template()?;
        self.expect(TokenKind::CloseCurlBrack)?;
        Ok(Conditional { condition, body })
    }

    #[token_context("Slot (slot(..))")]
    fn slot(&mut self) -> Result<Expression, PaxParseError> {
        self.expect_sequence([TokenKind::Slot, TokenKind::OpenParenth])?;
        let source = self.expression()?;
        self.expect(TokenKind::CloseParenth)?;
        Ok(source)
    }
}
