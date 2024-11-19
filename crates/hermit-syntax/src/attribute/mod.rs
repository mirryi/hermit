pub mod agent;
pub mod ensure;
pub mod forget;
pub mod have;

use paste::paste;
use proc_macro2::TokenStream;
use syn::parse::Parse;
use syn::Item;

macro_rules! parse_macro_input2 {
    ($tokenstream:ident as $ty:ty) => {
        match syn::parse::<$ty>($tokenstream) {
            Ok(data) => data,
            Err(err) => {
                return TokenStream::from(err.to_compile_error());
            }
        }
    };
    ($tokenstream:ident with $parser:path) => {
        match syn::parse::Parser::parse($parser, $tokenstream) {
            Ok(data) => data,
            Err(err) => {
                return TokenStream::from(err.to_compile_error());
            }
        }
    };
    ($tokenstream:ident) => {
        syn::parse_macro_input2!($tokenstream as _)
    };
}

macro_rules! impl_sig {
    ($name:ident, $variant:ident, $ty:ty) => {
        paste! {
            fn [<impl_ $name>](&self, args: Self::Args, item: $ty) -> TokenStream {
                self.impl_default(args, Item::$variant(item))
            }
        }
    };
    ($name:ident, $variant:ident) => {
        paste! { impl_sig!($name, $variant, syn::[<Item $variant>]); }
    };
}

pub trait ItemAttribute {
    type Args: Parse;

    fn impl_(&self, args: TokenStream, input: TokenStream) -> TokenStream {
        let args = args.into();
        let input = input.into();

        let args = parse_macro_input2!(args as Self::Args);
        let item = parse_macro_input2!(input as Item);

        match item {
            Item::Const(item) => self.impl_const(args, item),
            Item::Enum(item) => self.impl_enum(args, item),
            Item::ExternCrate(item) => self.impl_extern_crate(args, item),
            Item::Fn(item) => self.impl_fn(args, item),
            Item::ForeignMod(item) => self.impl_foreign_mod(args, item),
            Item::Impl(item) => self.impl_impl(args, item),
            Item::Macro(item) => self.impl_macro(args, item),
            Item::Mod(item) => self.impl_mod(args, item),
            Item::Static(item) => self.impl_static(args, item),
            Item::Struct(item) => self.impl_struct(args, item),
            Item::Trait(item) => self.impl_trait(args, item),
            Item::TraitAlias(item) => self.impl_trait_alias(args, item),
            Item::Type(item) => self.impl_type(args, item),
            Item::Union(item) => self.impl_union(args, item),
            Item::Use(item) => self.impl_use(args, item),
            Item::Verbatim(item) => self.impl_verbatim(args, item),
            _ => self.impl_wild(args, item),
        }
    }

    impl_sig!(fn, Fn);
    impl_sig!(const, Const);
    impl_sig!(enum, Enum);
    impl_sig!(extern_crate, ExternCrate);
    impl_sig!(foreign_mod, ForeignMod);
    impl_sig!(impl, Impl);
    impl_sig!(macro, Macro);
    impl_sig!(mod, Mod);
    impl_sig!(static, Static);
    impl_sig!(struct, Struct);
    impl_sig!(trait, Trait);
    impl_sig!(trait_alias, TraitAlias);
    impl_sig!(type, Type);
    impl_sig!(union, Union);
    impl_sig!(use, Use);
    impl_sig!(verbatim, Verbatim, TokenStream);

    fn impl_wild(&self, args: Self::Args, item: Item) -> TokenStream {
        self.impl_default(args, item)
    }

    fn impl_default(&self, _args: Self::Args, _item: Item) -> TokenStream {
        unimplemented!()
    }
}
