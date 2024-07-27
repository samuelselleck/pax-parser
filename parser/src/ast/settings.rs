use super::common::{Comment, FieldOrComment, Handler, Identifier};

#[derive(Debug)]
pub enum SettingsEntry {
    Comment(Comment),
    Handler(Handler),
    Class(Class),
    Id(Id),
}

#[derive(Debug)]
pub struct Class {
    pub name: Identifier,
    pub binding: Vec<FieldOrComment>,
}

#[derive(Debug)]
pub struct Id {
    pub name: Identifier,
    pub binding: Vec<FieldOrComment>,
}
