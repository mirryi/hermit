use proc_macro2::{Ident, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    ItemFn, Token,
};

use crate::user::ItemAttribute;
use crate::TOOL;

use super::form::Variable;

pub struct Attribute;

impl ItemAttribute for Attribute {
    type Args = Args;

    fn impl_fn(&self, args: Self::Args, item: ItemFn) -> TokenStream {
        let tool = TOOL.ident();
        let args = serde_json::to_string(&args).unwrap();

        quote! {
            #[#tool::forget(#args)]
            #item
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args {
    subject: Variable,
    dependencies: Vec<Variable>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        // (ex:) bar: foo, boo
        let subject = input.parse()?;
        let _ = input.parse::<Token![:]>()?;
        let dependencies = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?
            .iter()
            .map(Ident::to_string)
            .map(Variable)
            .collect();

        Ok(Self {
            subject,
            dependencies,
        })
    }
}
