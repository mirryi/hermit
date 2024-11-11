mod agent;
mod ensure;
mod forgets;
mod have;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn agent(args: TokenStream, input: TokenStream) -> TokenStream {
    agent::impl_(args, input)
}

#[proc_macro_attribute]
pub fn have(args: TokenStream, input: TokenStream) -> TokenStream {
    have::impl_(args, input)
}

#[proc_macro_attribute]
pub fn ensure(args: TokenStream, input: TokenStream) -> TokenStream {
    ensure::impl_(args, input)
}

#[proc_macro_attribute]
pub fn forgets(args: TokenStream, input: TokenStream) -> TokenStream {
    forgets::impl_(args, input)
}
