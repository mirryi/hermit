use std::mem;

use paste::paste;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{LitBool, Token};

use super::{Agent, Form, Variable};

#[derive(Debug, Clone)]
pub enum ParsedForm {
    Top,
    Bot,
    Prop(Variable),
    Neg(Box<Self>),
    Conj(Box<Self>, Box<Self>),
    Disj(Box<Self>, Box<Self>),
    Impl(Box<Self>, Box<Self>),
    BiImpl(Box<Self>, Box<Self>),
    Forall(Vec<Variable>, Box<Self>),
    Exist(Vec<Variable>, Box<Self>),
    K(Agent, Box<Self>),
    CK(Vec<Agent>, Box<Self>),
    DK(Vec<Agent>, Box<Self>),
}

impl Default for ParsedForm {
    fn default() -> Self {
        Self::Top
    }
}

impl From<ParsedForm> for Form {
    fn from(form: ParsedForm) -> Self {
        use epistemic::Form::*;

        macro_rules! from {
            ($e:expr) => {
                Box::new(Self::from(*$e).0)
            };
            ($($e:expr),+) => {
                vec![$(Self::from(*$e).0),+]
            };
        }

        match form {
            ParsedForm::Top => Self(Top),
            ParsedForm::Bot => Self(Bot),
            ParsedForm::Prop(x) => Self(Prop(x)),
            ParsedForm::Neg(p) => Self(Neg(from!(p))),
            ParsedForm::Conj(p1, p2) => Self(Conj(from!(p1, p2))),
            ParsedForm::Disj(p1, p2) => Self(Disj(from!(p1, p2))),
            ParsedForm::Impl(p1, p2) => Self(Impl(from!(p1), from!(p2))),
            ParsedForm::BiImpl(p1, p2) => Self(Equiv(from!(p1), from!(p2))),
            ParsedForm::Forall(bs, p) => Self(Forall(bs, from!(p))),
            ParsedForm::Exist(bs, p) => Self(Exist(bs, from!(p))),
            ParsedForm::K(ag, p) => Self(K(ag, from!(p))),
            ParsedForm::CK(ags, p) => Self(CK(ags, from!(p))),
            ParsedForm::DK(ags, p) => Self(DK(ags, from!(p))),
        }
    }
}

impl Parse for Agent {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(Self(ident.to_string()))
    }
}

impl Parse for Variable {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(Self(ident.to_string()))
    }
}

impl Parse for ParsedForm {
    fn parse(input: ParseStream) -> Result<Self> {
        parse(input, BP::MIN)
    }
}

fn parse(input: ParseStream, min_bp: BP) -> Result<ParsedForm> {
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

fn parse_parenthesized(input: ParseStream) -> Result<ParsedForm> {
    let inner;
    let _ = syn::parenthesized!(inner in input);

    parse(&inner, BP::MIN)
}

fn parse_top_bot(input: ParseStream) -> Result<ParsedForm> {
    // true
    // false
    match input.parse::<LitBool>()?.value() {
        true => Ok(ParsedForm::Top),
        false => Ok(ParsedForm::Bot),
    }
}

fn parse_forall(input: ParseStream) -> Result<ParsedForm> {
    // forall <var>, <var>, ... : <form>
    let _ = input.parse::<kw::forall>()?;
    let props = Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?
        .iter()
        .map(Ident::to_string)
        .map(Variable)
        .collect();
    let _ = input.parse::<Token![:]>()?;
    let form = input.parse()?;
    Ok(ParsedForm::Forall(props, Box::new(form)))
}

fn parse_exist(input: ParseStream) -> Result<ParsedForm> {
    // exist <var>, <var>, ... : <form>
    let _ = input.parse::<kw::exist>()?;
    let props = Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?
        .iter()
        .map(Ident::to_string)
        .map(Variable)
        .collect();
    let _ = input.parse::<Token![:]>()?;
    let form = input.parse()?;
    Ok(ParsedForm::Exist(props, Box::new(form)))
}

fn parse_know_<K>(input: ParseStream) -> Result<ParsedForm>
where
    K: Parse,
{
    // <K>[<ag> : <form>]
    let _ = input.parse::<K>()?;
    let inner;
    let _ = syn::bracketed!(inner in input);

    let agent = inner.parse()?;
    let _ = inner.parse::<Token![:]>()?;
    let form = parse(&inner, BP::MIN)?;

    Ok(ParsedForm::K(agent, Box::new(form)))
}

fn parse_know(input: ParseStream) -> Result<ParsedForm> {
    parse_know_::<kw::K>(input)
}

fn parse_common(input: ParseStream) -> Result<ParsedForm> {
    parse_know_::<kw::C>(input)
}

fn parse_distrib(input: ParseStream) -> Result<ParsedForm> {
    parse_know_::<kw::D>(input)
}

fn parse_variable(input: ParseStream) -> Result<ParsedForm> {
    let x = input.parse()?;
    Ok(ParsedForm::Prop(x))
}

macro_rules! parse_bin_rhs {
    ($name:ident, $token:tt, $variant:ident) => {
        paste! {
            fn [<parse_ $name>](
                input: ParseStream,
                min_bp: BP,
                lhs: ParsedForm,
            ) -> Result<Option<ParsedForm>> {
                let (lbp, rbp) = <Token![$token] as InfixBp>::infix_bp();
                if lbp < min_bp {
                    return Ok(None);
                }

                let rhs = parse(input, rbp)?;
                Ok(Some(ParsedForm::$variant(Box::new(lhs), Box::new(rhs))))
            }
        }
    };
}

parse_bin_rhs!(impl, ->, Impl);
parse_bin_rhs!(conj, &&, Conj);
parse_bin_rhs!(disj, ||, Disj);
parse_bin_rhs!(biimpl, ==, BiImpl);

fn parse_neg(input: ParseStream) -> Result<ParsedForm> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![!]) {
        // !<form>
        let _ = input.parse::<Token![!]>()?;
        let form = input.parse()?;
        Ok(ParsedForm::Neg(form))
    } else {
        Err(lookahead.error())
    }
}

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

impl InfixBp for Token![==] {
    fn infix_bp() -> (BP, BP) {
        (BP(3), BP(4))
    }
}

mod kw {
    syn::custom_keyword!(forall);
    syn::custom_keyword!(exist);
    syn::custom_keyword!(K);
    syn::custom_keyword!(C);
    syn::custom_keyword!(D);
}
