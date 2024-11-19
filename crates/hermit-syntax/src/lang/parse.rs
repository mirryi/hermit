use std::mem;

use hermit_core::{UntypedForm, UntypedRef};
use paste::paste;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token::Paren,
    LitBool, Token,
};

use super::{Agent, Form, Ident, LineColumn, Spanned, Variable};

impl Parse for Ident {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: proc_macro2::Ident = input.parse()?;
        let span = ident.span();
        Ok(Self(Spanned {
            value: ident.to_string(),
            range: span.byte_range(),
            start: span.start().into(),
            end: span.end().into(),
        }))
    }
}

impl From<proc_macro2::LineColumn> for LineColumn {
    fn from(lc: proc_macro2::LineColumn) -> Self {
        Self {
            line: lc.line,
            column: lc.column,
        }
    }
}

impl Parse for Agent {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl Parse for Variable {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl Parse for Form {
    fn parse(input: ParseStream) -> Result<Self> {
        let form = parse(input, BP::MIN)?;
        Ok(Self(form))
    }
}

fn parse(input: ParseStream, min_bp: BP) -> Result<UntypedForm<Ident, Ident>> {
    // parse lhs.
    let lookahead = input.lookahead1();
    let mut lhs = if lookahead.peek(Paren) {
        parse_parenthesized(input)?
    } else if lookahead.peek(LitBool) {
        parse_top_bot(input)?
    } else if lookahead.peek(Token![!]) {
        parse_neg(input)?
    } else if lookahead.peek(kw::forall) {
        parse_forall(input)?
    } else if lookahead.peek(kw::exist) {
        parse_exist(input)?
    } else if lookahead.peek(kw::agents) {
        parse_agents(input)?
    } else if lookahead.peek(kw::K) {
        parse_know(input)?
    } else if lookahead.peek(kw::C) {
        parse_common(input)?
    } else if lookahead.peek(kw::D) {
        parse_distrib(input)?
    } else if lookahead.peek(syn::Ident) {
        parse_variable(input)?
    } else {
        return Err(lookahead.error());
    };

    loop {
        let lookahead = input.lookahead1();
        lhs = if lookahead.peek(Token![->]) {
            match parse_impl(input, min_bp, mem::take(&mut lhs))? {
                Some(lhs) => lhs,
                None => break,
            }
        } else if lookahead.peek(Token![&&]) {
            match parse_conj(input, min_bp, mem::take(&mut lhs))? {
                Some(lhs) => lhs,
                None => break,
            }
        } else if lookahead.peek(Token![||]) {
            match parse_disj(input, min_bp, mem::take(&mut lhs))? {
                Some(lhs) => lhs,
                None => break,
            }
        } else if lookahead.peek(Token![^]) {
            match parse_xor(input, min_bp, mem::take(&mut lhs))? {
                Some(lhs) => lhs,
                None => break,
            }
        } else if lookahead.peek(Token![==]) {
            match parse_biimpl(input, min_bp, mem::take(&mut lhs))? {
                Some(lhs) => lhs,
                None => break,
            }
        } else {
            break;
        }
    }

    Ok(lhs)
}

fn parse_parenthesized(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    let inner;
    let _ = syn::parenthesized!(inner in input);

    parse(&inner, BP::MIN)
}

fn parse_top_bot(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // true
    // false
    match input.parse::<LitBool>()?.value() {
        true => Ok(UntypedForm::Top),
        false => Ok(UntypedForm::Bot),
    }
}

fn parse_forall(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // forall <var>, <var>, ... : <form>
    let _ = input.parse::<kw::forall>()?;
    let vars = Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?
        .into_iter()
        .collect();
    let _ = input.parse::<Token![:]>()?;
    let form = parse(input, BP::MIN)?;
    Ok(UntypedForm::Forall(vars, Box::new(form)))
}

fn parse_exist(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // exist <var>, <var>, ... : <form>
    let _ = input.parse::<kw::exist>()?;
    let vars = Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?
        .into_iter()
        .collect();
    let _ = input.parse::<Token![:]>()?;
    let form = parse(input, BP::MIN)?;
    Ok(UntypedForm::Exist(vars, Box::new(form)))
}

fn parse_agents(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // agents <ag> : <form>
    // agents <ag> in <ag>, <ag>, ... : <form>
    let _ = input.parse::<kw::agents>()?;
    let rf = input.parse()?;

    let lookahead = input.lookahead1();
    let ags = if lookahead.peek(Token![in]) {
        let _ = input.parse::<Token![in]>()?;
        Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?
            .into_iter()
            .collect()
    } else {
        vec![]
    };

    let _ = input.parse::<Token![:]>()?;
    let form = parse(input, BP::MIN)?;
    Ok(UntypedForm::ForG(rf, ags, Box::new(form)))
}

fn parse_know(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // K[<ag> : <form>]
    let _ = input.parse::<kw::K>()?;
    let inner;
    let _ = syn::bracketed!(inner in input);

    let ag = inner.parse::<Ident>()?;
    let _ = inner.parse::<Token![:]>()?;
    let form = parse(&inner, BP::MIN)?;

    Ok(UntypedForm::K(UntypedRef(ag), Box::new(form)))
}

fn parse_common(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // C[<ag>, <ag>, ... : <form>]
    let _ = input.parse::<kw::C>()?;
    let inner;
    let _ = syn::bracketed!(inner in input);

    let ags = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(&inner)?
        .into_iter()
        .map(UntypedRef)
        .collect();
    let _ = inner.parse::<Token![:]>()?;
    let form = parse(&inner, BP::MIN)?;

    Ok(UntypedForm::CK(ags, Box::new(form)))
}

fn parse_distrib(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    // D[<ag>, <ag>, ... : <form>]
    let _ = input.parse::<kw::D>()?;
    let inner;
    let _ = syn::bracketed!(inner in input);

    let ags = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(&inner)?
        .into_iter()
        .map(UntypedRef)
        .collect();
    let _ = inner.parse::<Token![:]>()?;
    let form = parse(&inner, BP::MIN)?;

    Ok(UntypedForm::DK(ags, Box::new(form)))
}

fn parse_variable(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    let x = input.parse()?;
    Ok(UntypedForm::Prop(x))
}

fn parse_neg(input: ParseStream) -> Result<UntypedForm<Ident, Ident>> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![!]) {
        // !<form>
        let _ = input.parse::<Token![!]>()?;
        let form = parse(input, BP::MIN)?;
        Ok(UntypedForm::Neg(Box::new(form)))
    } else {
        Err(lookahead.error())
    }
}

macro_rules! parse_bin_rhs {
    ($name:ident, $token:tt, $variant:ident) => {
        paste! {
            fn [<parse_ $name>](
                input: ParseStream,
                min_bp: BP,
                lhs: UntypedForm<Ident, Ident>,
            ) -> Result<Option<UntypedForm<Ident, Ident>>> {
                let (lbp, rbp) = <Token![$token] as InfixBp>::infix_bp();
                if lbp < min_bp {
                    return Ok(None);
                }

                let rhs = parse(input, rbp)?;
                Ok(Some(UntypedForm::$variant(Box::new(lhs), Box::new(rhs))))
            }
        }
    };
}

parse_bin_rhs!(impl, ->, Impl);
parse_bin_rhs!(conj, &&, Conj);
parse_bin_rhs!(disj, ||, Disj);
parse_bin_rhs!(xor, ^, Xor);
parse_bin_rhs!(biimpl, ==, BiImpl);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BP(pub u8);

impl BP {
    pub const MIN: BP = BP(0);
}

impl From<u8> for BP {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

trait InfixBp {
    fn infix_bp() -> (BP, BP);
}

impl InfixBp for Token![->] {
    fn infix_bp() -> (BP, BP) {
        (BP(2), BP(1))
    }
}

impl InfixBp for Token![&&] {
    fn infix_bp() -> (BP, BP) {
        (BP(5), BP(6))
    }
}

impl InfixBp for Token![||] {
    fn infix_bp() -> (BP, BP) {
        (BP(5), BP(6))
    }
}

impl InfixBp for Token![^] {
    fn infix_bp() -> (BP, BP) {
        (BP(5), BP(6))
    }
}

impl InfixBp for Token![==] {
    fn infix_bp() -> (BP, BP) {
        (BP(3), BP(4))
    }
}

mod kw {
    syn::custom_keyword!(forall);
    syn::custom_keyword!(exist);
    syn::custom_keyword!(agents);
    syn::custom_keyword!(K);
    syn::custom_keyword!(C);
    syn::custom_keyword!(D);
}
