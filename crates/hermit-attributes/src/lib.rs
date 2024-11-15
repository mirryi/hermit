use hermit_attributes_lib::user::{agent, ensure, forget, have, ItemAttribute};

macro_rules! attribute {
    ($name:ident) => {
        #[proc_macro_attribute]
        pub fn $name(
            args: proc_macro::TokenStream,
            input: proc_macro::TokenStream,
        ) -> proc_macro::TokenStream {
            $name::Attribute.impl_(args.into(), input.into()).into()
        }
    };
}

attribute!(agent);
attribute!(have);
attribute!(ensure);
attribute!(forget);
