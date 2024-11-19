use proc_macro2::TokenStream;
use quote::quote;
use rustc_ast::AttrItem;
use syn::{Ident, ItemFn};

use crate::{ToolItemAttribute, UserItemAttribute, TOOL};

pub struct UserAttribute;

impl UserItemAttribute for UserAttribute {
    type Args = TokenStream;

    fn impl_fn(&self, args: Self::Args, item: ItemFn) -> TokenStream {
        let attr = ToolAttribute::syntax(args);
        quote! {
            #attr
            #item
        }
    }
}

pub struct ToolAttribute {
    name: Ident,
}

impl ToolAttribute {
    const TOOL: &'static str = TOOL.name();
    const KIND: &'static str = "aaa";
}

impl ToolItemAttribute for ToolAttribute {
    fn syntax(args: TokenStream) -> TokenStream {
        let tool = TOOL.ident();
        quote! {
            #[#tool::agent(#args)]
        }
    }

    fn parse(attr: AttrItem) -> Option<Self> {
        let segments = &attr.path.segments;

        // ensure that the attribute is of the correct shape.
        let tool = segments.get(0)?.ident;
        let kind = segments.get(1)?.ident;
        let none = segments.get(2);
        if !(tool.as_str() == Self::TOOL && kind.as_str() == Self::KIND && none.is_none()) {
            return None;
        }

        // parse the arguments.
        let args = attr.args.inner_tokens().into();

        todo!()
    }
}
