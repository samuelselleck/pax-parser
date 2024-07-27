pub mod common;
pub mod expression;
pub mod settings;
pub mod template;

use self::{settings::SettingsEntry, template::TemplateEntry};

#[derive(Debug)]
pub struct PaxAst {
    pub templates: Vec<TemplateEntry>,
    pub settings: Vec<SettingsEntry>,
}
