use pax_parser_macros::token_context;

use crate::{
    ast::expression::{BinaryOp, Expression, Op, UnaryOp, Unit},
    lexer::TokenKind,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    /// Pratt parser for expressions
    #[token_context("Expression (var + 5.0/(i + 3%))")]
    pub fn literal_or_wrapped_expression(&mut self) -> Result<Expression, PaxParseError> {
        // Figure out if the {.. } wrapping this expression is intended
        // to be an expression like {5 + 4}, or a block/map, like {hello: 5}
        let is_expression = self.peek_token() == TokenKind::OpenCurlBrack && !self.is_map_next();

        Ok(if is_expression {
            self.expect(TokenKind::OpenCurlBrack)?;
            let expr = self.expression_with_min_bp(0)?;
            self.expect(TokenKind::CloseCurlBrack)?;
            expr
        } else {
            // parse iteral, and convert it into expression form
            let lit = self.literal()?;
            match lit.unit {
                Some(unit) => Expression::WithUnit {
                    val: Box::new(Expression::Value(lit.value)),
                    unit,
                },
                None => Expression::Value(lit.value),
            }
        })
    }

    pub fn expression(&mut self) -> Result<Expression, PaxParseError> {
        self.expression_with_min_bp(0)
    }

    fn expression_with_min_bp(&mut self, min_bp: u8) -> Result<Expression, PaxParseError> {
        let mut value = match self.peek_token() {
            TokenKind::Identifier
            | TokenKind::Integer
            | TokenKind::Float
            | TokenKind::OpenCurlBrack
            | TokenKind::String
            | TokenKind::OpenSquareBrack
            | TokenKind::OpenParenth => Expression::Value(self.value()?),
            TokenKind::Not => {
                let ((), rbp) = prefix_binding_power(UnaryOp::Not);
                self.tokens.next();
                let rhs = self.expression_with_min_bp(rbp)?;
                Expression::Unary {
                    op: UnaryOp::Not,
                    val: Box::new(rhs),
                }
            }
            TokenKind::Minus => {
                //prefix minus
                let ((), rbp) = prefix_binding_power(UnaryOp::Neg);
                self.tokens.next();
                let rhs = self.expression_with_min_bp(rbp)?;
                Expression::Unary {
                    op: UnaryOp::Neg,
                    val: Box::new(rhs),
                }
            }
            a => {
                eprintln!("found: {:?}", a);
                return Err(self.error([
                    TokenKind::Identifier,
                    TokenKind::Integer,
                    TokenKind::Float,
                    TokenKind::OpenCurlBrack,
                    TokenKind::String,
                    TokenKind::OpenSquareBrack,
                    TokenKind::OpenParenth,
                    TokenKind::Not,
                    TokenKind::Minus,
                ]));
            }
        };

        loop {
            let op = match self.peek_token() {
                // units
                TokenKind::Pixels => Op::Postfix(Unit::Pixels),
                TokenKind::Percent => Op::Postfix(Unit::Percent),
                TokenKind::Degrees => Op::Postfix(Unit::Degrees),
                TokenKind::Radians => Op::Postfix(Unit::Radians),

                // binary operators
                TokenKind::Plus => Op::Binary(BinaryOp::Add),
                TokenKind::Minus => Op::Binary(BinaryOp::Sub),
                TokenKind::Asterisk => Op::Binary(BinaryOp::Mult),
                TokenKind::Remainder => Op::Binary(BinaryOp::Mod),
                TokenKind::Range => Op::Binary(BinaryOp::Range),
                TokenKind::Eq => Op::Binary(BinaryOp::Eq),
                TokenKind::LessOrEq => Op::Binary(BinaryOp::LessOrEq),
                TokenKind::MoreOrEq => Op::Binary(BinaryOp::MoreOrEq),
                TokenKind::NotEq => Op::Binary(BinaryOp::NotEq),
                TokenKind::Or => Op::Binary(BinaryOp::Or),
                TokenKind::And => Op::Binary(BinaryOp::And),
                TokenKind::CloseAngBrack => Op::Binary(BinaryOp::LargerThan),
                TokenKind::OpenAngBrack => Op::Binary(BinaryOp::SmallerThan),
                TokenKind::Exp => Op::Binary(BinaryOp::Exp),
                TokenKind::Slash => {
                    if self.peek_nth_token(1) == TokenKind::CloseAngBrack {
                        // this slash is part of a closing tag not a
                        // div sign, return!
                        break;
                    }
                    Op::Binary(BinaryOp::Div)
                }
                _ => {
                    break;
                }
            };

            match op {
                Op::Binary(op) => {
                    let (lbp, rbp) = bin_binding_powers(&op);
                    if lbp < min_bp {
                        break;
                    }
                    //consume operator
                    self.tokens.next();
                    let rhs = self.expression_with_min_bp(rbp)?;

                    value = Expression::Binary {
                        left: Box::new(value),
                        op,
                        right: Box::new(rhs),
                    };
                }
                Op::Postfix(unit) => {
                    let (lbp, ()) = postfix_binding_power(unit);
                    if lbp < min_bp {
                        break;
                    }
                    //consume operator
                    self.tokens.next();
                    value = Expression::WithUnit {
                        val: Box::new(value),
                        unit,
                    };
                }
            }
        }
        Ok(value)
    }
}

fn bin_binding_powers(op: &BinaryOp) -> (u8, u8) {
    match op {
        BinaryOp::Or | BinaryOp::And => (1, 2),
        BinaryOp::NotEq
        | BinaryOp::LessOrEq
        | BinaryOp::MoreOrEq
        | BinaryOp::LargerThan
        | BinaryOp::SmallerThan
        | BinaryOp::Eq => (3, 4),
        BinaryOp::Add | BinaryOp::Sub => (5, 6),
        BinaryOp::Mult | BinaryOp::Div => (7, 8),
        BinaryOp::Mod => (9, 10),
        BinaryOp::Exp => (13, 14),
        BinaryOp::Range => (15, 16),
    }
}

fn prefix_binding_power(op: UnaryOp) -> ((), u8) {
    match op {
        UnaryOp::Neg | UnaryOp::Not => ((), 17),
    }
}

fn postfix_binding_power(unit: Unit) -> (u8, ()) {
    match unit {
        Unit::Pixels | Unit::Degrees | Unit::Radians | Unit::Percent => (19, ()),
    }
}
