use proc_macro2::{Ident, Span};

pub mod user;

pub struct Tool<'s> {
    name: &'s str,
}

impl<'s> Tool<'s> {
    pub const fn new(name: &'s str) -> Self {
        Self { name }
    }

    pub const fn name(&self) -> &'s str {
        self.name
    }

    pub fn ident(&self) -> Ident {
        Ident::new(self.name, Span::call_site())
    }
}

pub const TOOL: Tool = Tool::new("hermittool");
