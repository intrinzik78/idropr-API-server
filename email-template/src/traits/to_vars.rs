use std::collections::BTreeMap;

use crate::{
    enums::{Locale, Template, TemplateError},
    types::{RenderEngine,RenderedEmail}
};

type Result<T> = std::result::Result<T,TemplateError>;

pub trait ToVars {
    /// Convert the strongly-typed context into key/value strings for the renderer.
    fn to_vars(&self) -> BTreeMap<&'static str, String>;
}

pub trait EmailTemplate: ToVars {
    /// load the template by Template::variant, define on the email struct itself
    fn variant() -> Template;

    /// expose the render engine to the email
    fn render(&self, local: &Locale) -> Result<RenderedEmail> {
        RenderEngine::render(Self::variant(), local, &self.to_vars())
    }
}