use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    ItemFn, Token,
};

use crate::lang::Variable;
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
    subject: Variable,
    dependencies: Vec<Variable>,
}

impl Meta {
    pub const KIND: &'static str = "forget";
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> Result<Self> {
        // (ex:) bar: foo, boo
        let subject = input.parse()?;
        let _ = input.parse::<Token![:]>()?;
        let dependencies = Punctuated::<Variable, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect();

        Ok(Self {
            subject,
            dependencies,
        })
    }
}
