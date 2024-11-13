use proc_macro::TokenStream;

use hermit_attributes_lib::{agent, ensure, forgets, have, UserItemAttribute};

macro_rules! attribute {
    ($name:ident) => {
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, input: TokenStream) -> TokenStream {
            $name::UserAttribute.impl_(args.into(), input.into()).into()
        }
    };
}

attribute!(agent);
attribute!(have);
attribute!(ensure);
attribute!(forgets);
