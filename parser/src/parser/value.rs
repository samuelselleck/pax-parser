use pax_parser_macros::token_context;

use crate::{
    ast::{
        common::Identifier,
        expression::{EnumVariant, Expression, FunctionCall, Value},
    },
    lexer::Token,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Value (5px, {..})")]
    pub fn value(&mut self) -> Result<Value, PaxParseError> {
        Ok(match self.tokens.peek() {
            Token::Integer => Value::Int(self.tokens.next().span),
            Token::Float => Value::Float(self.tokens.next().span),
            Token::Identifier => match self.tokens.peek_nth(1) {
                Token::OpenParenth => Value::FunctionCall(self.function_call()?),
                Token::PathSep => Value::EnumVariant(self.enum_variant()?),
                Token::OpenCurlBrack if self.is_map_next() => Value::Object(self.object()?),
                _ => Value::Variable(self.variable()?),
            },
            Token::String => Value::String(self.tokens.next().span),
            Token::OpenCurlBrack => Value::Object(self.object()?),
            Token::OpenSquareBrack => Value::List(
                self.sequence_enclosed_in(Token::OpenSquareBrack, Token::CloseSquareBrack)?,
            ),
            Token::OpenParenth => {
                Value::Tuple(self.sequence_enclosed_in(Token::OpenParenth, Token::CloseParenth)?)
            }
            _ => {
                return Err(self.error([
                    Token::Integer,
                    Token::Float,
                    Token::Identifier,
                    Token::String,
                    Token::OpenCurlBrack,
                    Token::OpenSquareBrack,
                    Token::OpenParenth,
                ]));
            }
        })
    }

    #[token_context("Variable")]
    fn variable(&mut self) -> Result<Vec<Identifier>, PaxParseError> {
        let mut var_path = Vec::new();
        loop {
            let ident = self.expect(Token::Identifier)?;
            var_path.push(Identifier(ident.span));
            if self.tokens.next_if(|t| t == Token::Period).is_none() {
                break;
            }
        }
        Ok(var_path)
    }

    fn enum_variant(&mut self) -> Result<EnumVariant, PaxParseError> {
        let [name, _, variant] =
            self.expect_sequence([Token::Identifier, Token::PathSep, Token::Identifier])?;
        let args = if self.tokens.peek() == Token::OpenParenth {
            self.sequence_enclosed_in(Token::OpenParenth, Token::CloseParenth)?
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
        let ident = self.expect(Token::Identifier)?;
        Ok(FunctionCall {
            name: Identifier(ident.span),
            arguments: self.sequence_enclosed_in(Token::OpenParenth, Token::CloseParenth)?,
        })
    }

    #[token_context("Sequence ([foo, 5px], or (foo, 5px))")]
    fn sequence_enclosed_in(
        &mut self,
        open: Token,
        close: Token,
    ) -> Result<Vec<Expression>, PaxParseError> {
        self.expect(open)?;
        let mut entries = vec![];
        loop {
            if self.tokens.next_if(|t| t == close).is_some() {
                break;
            }
            entries.push(self.expression()?);
            self.tokens.next_if(|t| t == Token::Comma);
        }
        Ok(entries)
    }
}
