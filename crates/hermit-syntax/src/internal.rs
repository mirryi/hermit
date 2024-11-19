pub mod agent;
pub mod ensure;
pub mod forgets;
pub mod have;

use rustc_ast::AttrItem;

use proc_macro2::{Span, TokenStream};
use syn::Ident;

pub trait ToolItemAttribute: Sized {
    fn syntax(args: TokenStream) -> TokenStream;

    fn parse(attr: AttrItem) -> Option<Self>;
}

pub struct ToolNamespace<'s> {
    name: &'s str,
}

impl<'s> ToolNamespace<'s> {
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

pub const TOOL: ToolNamespace = ToolNamespace::new("hermittool");
