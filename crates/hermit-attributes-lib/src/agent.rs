use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::UserItemAttribute;

pub struct UserAttribute;

impl UserItemAttribute for UserAttribute {
    type Args = TokenStream;

    fn impl_fn(&self, args: Self::Args, item: ItemFn) -> TokenStream {
        quote! {
            #[hermittool::agent(#args)]
            #item
        }
    }
}
