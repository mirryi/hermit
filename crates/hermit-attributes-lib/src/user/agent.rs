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

use super::form::Agent;

pub struct Attribute;

impl ItemAttribute for Attribute {
    type Args = Args;

    fn impl_fn(&self, args: Self::Args, item: ItemFn) -> TokenStream {
        let tool = TOOL.ident();
        let args = serde_json::to_string(&args).unwrap();

        quote! {
            #[#tool::agent(#args)]
            #item
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args {
    names: Vec<Agent>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse list of agent names as comma-separated identifiers.
        let names = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(input)?
            .iter()
            .map(Ident::to_string)
            .map(Agent)
            .collect();
        Ok(Self { names })
    }
}
