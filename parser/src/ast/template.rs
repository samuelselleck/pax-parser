use super::{
    common::{Comment, Field, Handler, Identifier},
    expression::Expression,
};

#[derive(Debug)]
pub enum TemplateEntry {
    Comment(Comment),
    Tag(Tag),
    Loop(Loop),
    Conditional(Conditional),
    Slot(Expression),
}

#[derive(Debug)]
pub struct Loop {
    pub pattern: MatchPattern,
    pub source: Expression,
    pub body: Vec<TemplateEntry>,
}

#[derive(Debug)]
pub struct Conditional {
    pub condition: Expression,
    pub body: Vec<TemplateEntry>,
}

#[derive(Debug)]
pub struct Tag {
    pub name: Identifier,
    pub attributes: Vec<Attribute>,
    pub body: Vec<TemplateEntry>,
}

#[derive(Debug)]
pub enum Attribute {
    Handler(Handler),
    Field(Field),
    Binding(Binding),
}

#[derive(Debug)]
pub struct Binding {
    pub key: Identifier,
    pub value: Identifier,
}

#[derive(Debug)]
pub enum MatchPattern {
    Identifier(Identifier),
    Tuple(Identifier, Identifier),
}
