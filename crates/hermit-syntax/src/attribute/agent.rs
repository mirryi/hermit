use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    ItemFn, Token,
};

use crate::lang::Agent;
use crate::TOOL;

use super::ItemAttribute;

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
        let names = Punctuated::<Agent, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect();
        Ok(Self { names })
    }
}
