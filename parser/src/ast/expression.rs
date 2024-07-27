use crate::lexer::Span;

use super::common::{FieldOrComment, Identifier};

#[derive(Debug)]
pub enum Expression {
    Value(Value),
    Unary {
        op: UnaryOp,
        val: Box<Expression>,
    },
    WithUnit {
        val: Box<Expression>,
        unit: Unit,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
}

// A literal contains a very constrained subset of all possible expressions,
// see literal parsing for what it can contain
#[derive(Debug)]
pub struct Literal {
    pub value: Value,
    pub unit: Option<Unit>,
}

#[derive(Debug)]
pub enum Value {
    Variable(Vec<Identifier>),
    EnumVariant(EnumVariant),
    Float(Span),
    Int(Span),
    String(Span),
    Object(Object),
    FunctionCall(FunctionCall),
    List(Vec<Expression>),
    Tuple(Vec<Expression>),
}

#[derive(Debug)]
pub struct Object {
    pub name: Option<Identifier>,
    pub fields: Vec<FieldOrComment>,
}

#[derive(Debug)]
pub struct EnumVariant {
    pub name: Identifier,
    pub variant: Identifier,
    pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct FunctionCall {
    pub name: Identifier,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Binary(BinaryOp),
    Postfix(Unit),
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    Degrees,
    Radians,
    Percent,
    Pixels,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,         // +
    Sub,         // -
    Mult,        // *
    Div,         // /
    Mod,         // %%
    Range,       // ..
    Eq,          // ==
    NotEq,       // !=
    LessOrEq,    // <=
    MoreOrEq,    // >=
    LargerThan,  // >
    SmallerThan, //<
    Or,          // ||
    And,         // &&
    Exp,         // ^
}
