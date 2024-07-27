use pax_parser_macros::token_context;

use crate::{
    ast::expression::{Literal, Unit, Value},
    lexer::Token,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    #[token_context("Literal (var, 5px, [...], {..})")]
    pub fn literal(&mut self) -> Result<Literal, PaxParseError> {
        // most literals are just the same as values allowed in an expression.
        // The only possible addition is a postfix unit for floats and ints.
        let value = self.value()?;

        let is_numeric = matches!(&value, Value::Int(_) | Value::Float(_));
        let next_is_unit = matches!(
            self.tokens.peek(),
            Token::Pixels | Token::Percent | Token::Radians | Token::Degrees,
        );

        let unit = if is_numeric && next_is_unit {
            Some(match self.tokens.next().token_type {
                Token::Pixels => Unit::Pixels,
                Token::Percent => Unit::Percent,
                Token::Radians => Unit::Radians,
                Token::Degrees => Unit::Degrees,
                _ => unreachable!(),
            })
        } else {
            None
        };

        Ok(Literal { value, unit })
    }
}
