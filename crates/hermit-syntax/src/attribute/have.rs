use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{
    parse::{Parse, ParseStream, Result},
    ItemFn,
};

use crate::lang::Form;
use crate::TOOL;

use super::ItemAttribute;

pub struct Attribute;

impl ItemAttribute for Attribute {
    type Args = Args;

    fn impl_fn(&self, args: Self::Args, item: ItemFn) -> TokenStream {
        let tool = TOOL.ident();
        let args = serde_json::to_string(&args).unwrap();

        quote! {
            #[#tool::have(#args)]
            #item
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Args {
    form: Form,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let form = input.parse()?;
        Ok(Self { form })
    }
}