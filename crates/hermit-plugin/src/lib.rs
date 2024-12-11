#![feature(rustc_private)]

extern crate either;
extern crate rustc_ast;
extern crate rustc_borrowck;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_lexer;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;

mod plugin;

mod analyse;
mod collect;
mod meta;

pub use plugin::HermitPlugin;
