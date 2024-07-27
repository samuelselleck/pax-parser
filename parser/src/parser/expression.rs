use pax_parser_macros::token_context;

use crate::{
    ast::expression::{BinaryOp, Expression, Op, UnaryOp, Unit},
    lexer::Token,
};

use super::{Parser, PaxParseError};

impl<'src> Parser<'src> {
    /// Pratt parser for expressions
    #[token_context("Expression (var + 5.0/(i + 3%))")]
    pub fn literal_or_wrapped_expression(&mut self) -> Result<Expression, PaxParseError> {
        // Figure out if the {.. } wrapping this expression is intended
        // to be an expression like {5 + 4}, or a block/map, like {hello: 5}
        let is_expression = self.tokens.peek() == Token::OpenCurlBrack && !self.is_map_next();

        Ok(if is_expression {
            self.expect(Token::OpenCurlBrack)?;
            let expr = self.expression_with_min_bp(0)?;
            self.expect(Token::CloseCurlBrack)?;
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
        let mut value = match self.tokens.peek() {
            Token::Identifier
            | Token::Integer
            | Token::Float
            | Token::OpenCurlBrack
            | Token::String
            | Token::OpenSquareBrack
            | Token::OpenParenth => Expression::Value(self.value()?),
            Token::Not => {
                let ((), rbp) = prefix_binding_power(UnaryOp::Not);
                self.tokens.next();
                let rhs = self.expression_with_min_bp(rbp)?;
                Expression::Unary {
                    op: UnaryOp::Not,
                    val: Box::new(rhs),
                }
            }
            Token::Minus => {
                //prefix minus
                let ((), rbp) = prefix_binding_power(UnaryOp::Neg);
                self.tokens.next();
                let rhs = self.expression_with_min_bp(rbp)?;
                Expression::Unary {
                    op: UnaryOp::Neg,
                    val: Box::new(rhs),
                }
            }
            _ => {
                return Err(self.error([
                    Token::Identifier,
                    Token::Integer,
                    Token::Float,
                    Token::OpenCurlBrack,
                    Token::String,
                    Token::OpenSquareBrack,
                    Token::OpenParenth,
                    Token::Not,
                    Token::Minus,
                ]));
            }
        };

        loop {
            let op = match self.tokens.peek() {
                // units
                Token::Pixels => Op::Postfix(Unit::Pixels),
                Token::Percent => Op::Postfix(Unit::Percent),
                Token::Degrees => Op::Postfix(Unit::Degrees),
                Token::Radians => Op::Postfix(Unit::Radians),

                // binary operators
                Token::Plus => Op::Binary(BinaryOp::Add),
                Token::Minus => Op::Binary(BinaryOp::Sub),
                Token::Asterisk => Op::Binary(BinaryOp::Mult),
                Token::Remainder => Op::Binary(BinaryOp::Mod),
                Token::Range => Op::Binary(BinaryOp::Range),
                Token::Eq => Op::Binary(BinaryOp::Eq),
                Token::LessOrEq => Op::Binary(BinaryOp::LessOrEq),
                Token::MoreOrEq => Op::Binary(BinaryOp::MoreOrEq),
                Token::NotEq => Op::Binary(BinaryOp::NotEq),
                Token::Or => Op::Binary(BinaryOp::Or),
                Token::And => Op::Binary(BinaryOp::And),
                Token::CloseAngBrack => Op::Binary(BinaryOp::LargerThan),
                Token::OpenAngBrack => Op::Binary(BinaryOp::SmallerThan),
                Token::Exp => Op::Binary(BinaryOp::Exp),
                Token::Slash => {
                    if self.tokens.peek_nth(1) == Token::CloseAngBrack {
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
