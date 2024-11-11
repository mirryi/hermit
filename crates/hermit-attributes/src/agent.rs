use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{parse::Result, parse_macro_input};
use syn::{Item, ItemFn};

#[derive(Debug)]
struct Agent {
    name: Ident,
}

impl Parse for Agent {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        Ok(Self { name })
    }
}

pub fn impl_(args: TokenStream, input: TokenStream) -> TokenStream {
    let agent = parse_macro_input!(args as Agent);
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Const(_) => unimplemented!(),
        Item::Enum(_) => unimplemented!(),
        Item::ExternCrate(_) => unimplemented!(),
        Item::Fn(fun) => impl_fn(agent, fun),
        Item::ForeignMod(_) => unimplemented!(),
        Item::Impl(_) => unimplemented!(),
        Item::Macro(_) => unimplemented!(),
        Item::Mod(_) => unimplemented!(),
        Item::Static(_) => unimplemented!(),
        Item::Struct(_) => unimplemented!(),
        Item::Trait(_) => unimplemented!(),
        Item::TraitAlias(_) => unimplemented!(),
        Item::Type(_) => unimplemented!(),
        Item::Union(_) => unimplemented!(),
        Item::Use(_) => unimplemented!(),
        Item::Verbatim(_) => unimplemented!(),
        _ => unimplemented!(),
    }
}

fn impl_fn(_agent: Agent, fun: ItemFn) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = fun;

    ItemFn {
        attrs,
        vis,
        sig,
        block,
    }
    .to_token_stream()
    .into()
}
