use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::{Comment, Field, Handler, Identifier},
        expression::Expression,
        template::{Attribute, Binding, Conditional, Loop, MatchPattern, Tag, TemplateEntry},
    },
    lexer::Token,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Template")]
    pub fn template(&mut self) -> Result<Vec<TemplateEntry>, PaxParseError> {
        let mut template: Vec<TemplateEntry> = vec![];
        loop {
            let entry = match self.tokens.peek() {
                Token::CloseCurlBrack | Token::AtSymbol | Token::EOF => break,
                Token::OpenAngBrack => {
                    if self.tokens.peek_nth(1) == Token::Slash {
                        break;
                    } else {
                        TemplateEntry::Tag(self.tag()?)
                    }
                }
                Token::For => TemplateEntry::Loop(self.for_loop()?),
                Token::If => TemplateEntry::Conditional(self.condition()?),
                Token::Slot => TemplateEntry::Slot(self.slot()?),
                Token::Comment => {
                    TemplateEntry::Comment(Comment(self.expect(Token::Comment)?.span))
                }
                _ => {
                    return Err(self.error([
                        Token::CloseCurlBrack,
                        Token::OpenAngBrack,
                        Token::For,
                        Token::If,
                        Token::Slot,
                        Token::Comment,
                    ]))
                }
            };
            template.push(entry);
        }
        Ok(template)
    }

    #[token_context("Tag pair (<tag>..</tag>)")]
    fn tag(&mut self) -> Result<Tag, PaxParseError> {
        self.expect(Token::OpenAngBrack)?;
        let name = self.expect(Token::Identifier)?;

        let mut attributes = vec![];
        loop {
            match self.tokens.peek() {
                Token::CloseAngBrack | Token::Slash => break,
                _ => {
                    attributes.push(self.attribute()?);
                }
            }
        }
        let body = match self.tokens.peek() {
            Token::CloseAngBrack => {
                self.tokens.next();
                let template = self.template()?;
                let [_, _, ident, _] = self.expect_sequence([
                    Token::OpenAngBrack,
                    Token::Slash,
                    Token::Identifier,
                    Token::CloseAngBrack,
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
            Token::Slash => {
                self.tokens.next();
                self.expect(Token::CloseAngBrack)?;
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
        Ok(match self.tokens.peek() {
            Token::AtSymbol => {
                let [_, key, _] =
                    self.expect_sequence([Token::AtSymbol, Token::Identifier, Token::Assign])?;
                Attribute::Handler(Handler {
                    key: Identifier(key.span),
                    value: Identifier(self.expect(Token::Identifier)?.span),
                })
            }
            Token::Bind => {
                let [_, _, key, _, value] = self.expect_sequence([
                    Token::Bind,
                    Token::Colon,
                    Token::Identifier,
                    Token::Assign,
                    Token::Identifier,
                ])?;
                Attribute::Binding(Binding {
                    key: Identifier(key.span),
                    value: Identifier(value.span),
                })
            }
            Token::Identifier => {
                let [key, _] = self.expect_sequence([Token::Identifier, Token::Assign])?;
                Attribute::Field(Field {
                    key: Identifier(key.span),
                    value: self.literal_or_wrapped_expression()?,
                })
            }
            _ => return Err(self.error([Token::AtSymbol, Token::Bind, Token::Identifier])),
        })
    }

    #[token_context("For loop (for i in items {..})")]
    fn for_loop(&mut self) -> Result<Loop, PaxParseError> {
        self.expect(Token::For)?;

        let pattern = if self.tokens.peek() == Token::OpenParenth {
            let [_, t1, _, t2, _] = self.expect_sequence([
                Token::OpenParenth,
                Token::Identifier,
                Token::Comma,
                Token::Identifier,
                Token::CloseParenth,
            ])?;
            MatchPattern::Tuple(Identifier(t1.span), Identifier(t2.span))
        } else {
            let ident = self.expect(Token::Identifier)?;
            MatchPattern::Identifier(Identifier(ident.span))
        };

        self.expect(Token::In)?;
        let source = self.expression()?;
        self.expect(Token::OpenCurlBrack)?;
        let body = self.template()?;
        self.expect(Token::CloseCurlBrack)?;
        Ok(Loop {
            pattern,
            source,
            body,
        })
    }

    #[token_context("Condition (if cond {..})")]
    fn condition(&mut self) -> Result<Conditional, PaxParseError> {
        self.expect(Token::If)?;
        let condition = self.expression()?;
        self.expect(Token::OpenCurlBrack)?;
        let body = self.template()?;
        self.expect(Token::CloseCurlBrack)?;
        Ok(Conditional { condition, body })
    }

    #[token_context("Slot (slot(..))")]
    fn slot(&mut self) -> Result<Expression, PaxParseError> {
        self.expect_sequence([Token::Slot, Token::OpenParenth])?;
        let source = self.expression()?;
        self.expect(Token::CloseParenth)?;
        Ok(source)
    }
}
