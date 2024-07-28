use pax_parser_macros::token_context;

use crate::{
    ast::expression::{Literal, Unit, Value},
    lexer::TokenKind,
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
            self.peek_token(),
            TokenKind::Pixels | TokenKind::Percent | TokenKind::Radians | TokenKind::Degrees,
        );

        let unit = if is_numeric && next_is_unit {
            Some(match self.next_token().kind {
                TokenKind::Pixels => Unit::Pixels,
                TokenKind::Percent => Unit::Percent,
                TokenKind::Radians => Unit::Radians,
                TokenKind::Degrees => Unit::Degrees,
                _ => unreachable!("already checked with if above"),
            })
        } else {
            None
        };

        Ok(Literal { value, unit })
    }
}
