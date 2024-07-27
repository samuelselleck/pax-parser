use crate::lexer::Span;

use super::expression::Expression;

#[derive(Debug)]
pub enum FieldOrComment {
    Field(Field),
    Comment(Comment),
}

#[derive(Debug)]
pub struct Identifier(pub Span);

#[derive(Debug)]
pub struct Field {
    pub key: Identifier,
    pub value: Expression,
}

#[derive(Debug)]
pub struct Handler {
    pub key: Identifier,
    pub value: Identifier,
}

#[derive(Debug)]
pub struct Comment(pub Span);
