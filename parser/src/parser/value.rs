use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::Identifier,
        expression::{EnumVariant, Expression, FunctionCall, Value},
    },
    lexer::TokenKind,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Value (5px, {..})")]
    pub fn value(&mut self) -> Result<Value, PaxParseError> {
        Ok(match self.peek_token() {
            TokenKind::Integer => Value::Int(self.next_token().span),
            TokenKind::Float => Value::Float(self.next_token().span),
            TokenKind::Identifier => match self.peek_nth_token(1) {
                TokenKind::OpenParenth => Value::FunctionCall(self.function_call()?),
                TokenKind::PathSep => Value::EnumVariant(self.enum_variant()?),
                TokenKind::OpenCurlBrack if self.is_map_next() => Value::Object(self.object()?),
                _ => Value::Variable(self.variable()?),
            },
            TokenKind::String => Value::String(self.next_token().span),
            TokenKind::OpenCurlBrack => Value::Object(self.object()?),
            TokenKind::OpenSquareBrack => Value::List(
                self.sequence_enclosed_in(TokenKind::OpenSquareBrack, TokenKind::CloseSquareBrack)?,
            ),
            TokenKind::OpenParenth => Value::Tuple(
                self.sequence_enclosed_in(TokenKind::OpenParenth, TokenKind::CloseParenth)?,
            ),
            _ => {
                return Err(self.error([
                    TokenKind::Integer,
                    TokenKind::Float,
                    TokenKind::Identifier,
                    TokenKind::String,
                    TokenKind::OpenCurlBrack,
                    TokenKind::OpenSquareBrack,
                    TokenKind::OpenParenth,
                ]));
            }
        })
    }

    #[token_context("Variable")]
    fn variable(&mut self) -> Result<Vec<Identifier>, PaxParseError> {
        let mut var_path = Vec::new();
        loop {
            let ident = self.expect(TokenKind::Identifier)?;
            var_path.push(Identifier(ident.span));
            if self.next_token_if(|t| t == TokenKind::Period).is_none() {
                break;
            }
        }
        Ok(var_path)
    }

    fn enum_variant(&mut self) -> Result<EnumVariant, PaxParseError> {
        let [name, _, variant] = self.expect_sequence([
            TokenKind::Identifier,
            TokenKind::PathSep,
            TokenKind::Identifier,
        ])?;
        let args = if self.peek_token() == TokenKind::OpenParenth {
            self.sequence_enclosed_in(TokenKind::OpenParenth, TokenKind::CloseParenth)?
        } else {
            vec![]
        };
        Ok(EnumVariant {
            name: Identifier(name.span),
            variant: Identifier(variant.span),
            arguments: args,
        })
    }

    #[token_context("Function call")]
    fn function_call(&mut self) -> Result<FunctionCall, PaxParseError> {
        let ident = self.expect(TokenKind::Identifier)?;
        Ok(FunctionCall {
            name: Identifier(ident.span),
            arguments: self
                .sequence_enclosed_in(TokenKind::OpenParenth, TokenKind::CloseParenth)?,
        })
    }

    #[token_context("Sequence ([foo, 5px], or (foo, 5px))")]
    fn sequence_enclosed_in(
        &mut self,
        open: TokenKind,
        close: TokenKind,
    ) -> Result<Vec<Expression>, PaxParseError> {
        self.expect(open)?;
        let mut entries = vec![];
        loop {
            if self.next_token_if(|t| t == close).is_some() {
                break;
            }
            entries.push(self.expression()?);
            self.next_token_if(|t| t == TokenKind::Comma);
        }
        Ok(entries)
    }
}
