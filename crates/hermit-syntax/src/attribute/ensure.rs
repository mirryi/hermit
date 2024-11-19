use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{
    parse::{Parse, ParseStream, Result},
    ItemFn,
};

use crate::lang::Form;
use crate::TOOL;

use super::{Encode, ItemAttribute};

pub struct Attribute;

impl ItemAttribute for Attribute {
    type Args = Meta;

    fn impl_fn(&self, args: Self::Args, item: ItemFn) -> TokenStream {
        let tool = TOOL.ident();
        let kind = Ident::new(Meta::KIND, Span::call_site());
        let args = args.encode();

        quote! {
            #[#tool::#kind(#args)]
            #item
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    form: Form,
}

impl Meta {
    pub const KIND: &'static str = "have";
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> Result<Self> {
        let form = input.parse()?;
        Ok(Self { form })
    }
}
