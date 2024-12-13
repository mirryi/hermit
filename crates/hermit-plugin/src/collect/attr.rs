use std::{collections::BTreeMap, iter};

use hermit_core::{UntypedForm, UntypedRef};
use rustc_ast::{
    token::{Lit, LitKind, Token, TokenKind},
    tokenstream::TokenTree,
    AttrKind, Attribute,
};
use rustc_lexer::unescape;

use hermit_syntax::{
    attribute::{
        AgentMeta as AgentAttribute, Decode as DecodeAttribute, EnsureMeta as EnsureAttribute,
        ForgetMeta as ForgetAttribute, HaveMeta as HaveAttribute,
    },
    TOOL,
};

use crate::meta;

pub struct AttrCollector<'a> {
    attr: &'a Attribute,
}

impl<'a> AttrCollector<'a> {
    pub fn new(attr: &'a Attribute) -> Self {
        Self { attr }
    }

    pub fn collect(&self) -> Option<AttrInfo> {
        let normal = match &self.attr.kind {
            AttrKind::Normal(normal) => normal,
            AttrKind::DocComment(_, _) => return None,
        };

        let segments = &normal.item.path.segments;
        let tool = segments.first()?.ident.as_str();
        let kind = segments.get(1)?.ident.as_str();
        let none = segments.get(2);
        if !(tool == TOOL.name() && none.is_none()) {
            return None;
        }

        // extract the string argument.
        let mut args = normal.item.args.inner_tokens().into_trees();
        let arg = match args.next_ref().unwrap() {
            TokenTree::Token(
                Token {
                    kind:
                        TokenKind::Literal(Lit {
                            kind: LitKind::Str,
                            symbol,
                            suffix: _,
                        }),
                    span: _,
                },
                _,
            ) => {
                let mut buf = String::new();
                unescape::unescape_literal(symbol.as_str(), unescape::Mode::Str, &mut |_, c| {
                    buf.push(c.unwrap())
                });
                buf
            }
            _ => panic!(),
        };

        if kind == AgentAttribute::KIND {
            Some(self.collect_agent(&arg))
        } else if kind == HaveAttribute::KIND {
            Some(self.collect_have(&arg))
        } else if kind == EnsureAttribute::KIND {
            Some(self.collect_ensure(&arg))
        } else if kind == ForgetAttribute::KIND {
            Some(self.collect_forget(&arg))
        } else {
            panic!()
        }
    }

    fn collect_agent(&self, arg: &str) -> AttrInfo {
        AttrInfo::Agent(AgentAttribute::decode(arg))
    }

    fn collect_have(&self, arg: &str) -> AttrInfo {
        AttrInfo::Have(HaveAttribute::decode(arg))
    }

    fn collect_ensure(&self, arg: &str) -> AttrInfo {
        AttrInfo::Ensure(EnsureAttribute::decode(arg))
    }

    fn collect_forget(&self, arg: &str) -> AttrInfo {
        AttrInfo::Forget(ForgetAttribute::decode(arg))
    }
}

#[derive(Debug, Clone)]
pub enum AttrInfo {
    Agent(AgentAttribute),
    Have(HaveAttribute),
    Ensure(EnsureAttribute),
    Forget(ForgetAttribute),
}

impl AttrInfo {
    pub fn variables(&self) -> impl Iterator<Item = &hermit_syntax::lang::Ident> {
        let iter: Box<dyn Iterator<Item = _>> = match self {
            AttrInfo::Agent(_) => Box::new(iter::empty()),
            AttrInfo::Have(HaveAttribute { form }) => Box::new(form.0.vocab()),
            AttrInfo::Ensure(EnsureAttribute { form }) => Box::new(form.0.vocab()),
            AttrInfo::Forget(ForgetAttribute {
                subject,
                dependencies,
            }) => Box::new(iter::once(subject).chain(dependencies).map(|var| &var.0)),
        };
        iter
    }

    pub fn have_to_meta(
        attr: HaveAttribute,
        vars: &BTreeMap<String, meta::FunctionLocation>,
    ) -> meta::HaveAnn {
        meta::HaveAnn {
            form: trans_form(attr.form.0, vars),
        }
    }

    pub fn ensure_to_meta(
        attr: EnsureAttribute,
        vars: &BTreeMap<String, meta::FunctionLocation>,
    ) -> meta::EnsureAnn {
        meta::EnsureAnn {
            form: trans_form(attr.form.0, vars),
        }
    }

    pub fn forget_to_meta(
        attr: ForgetAttribute,
        vars: &BTreeMap<String, meta::FunctionLocation>,
    ) -> meta::ForgetAnn {
        // TODO replace these unwraps with some nice error handling
        let subject = meta::LocalTarget::Local(*vars.get(&attr.subject.0 .0.value).unwrap());
        let dependencies = attr
            .dependencies
            .into_iter()
            .map(|dep| *vars.get(&dep.0 .0.value).unwrap())
            .map(meta::LocalTarget::Local)
            .collect();
        meta::ForgetAnn {
            subject,
            dependencies,
        }
    }
}

fn trans_form(
    p: UntypedForm<hermit_syntax::attribute::Ident, hermit_syntax::attribute::Ident>,
    vars: &BTreeMap<String, meta::FunctionLocation>,
) -> UntypedForm<meta::Agent, meta::LocalTarget> {
    match p {
        UntypedForm::Top => UntypedForm::Top,
        UntypedForm::Bot => UntypedForm::Bot,
        UntypedForm::Prop(b) => {
            UntypedForm::Prop(meta::LocalTarget::Local(*vars.get(&b.0.value).unwrap()))
        }
        UntypedForm::Neg(p) => UntypedForm::Neg(Box::new(trans_form(*p, vars))),
        UntypedForm::Conj(p1, p2) => UntypedForm::Conj(
            Box::new(trans_form(*p1, vars)),
            Box::new(trans_form(*p2, vars)),
        ),
        UntypedForm::Disj(p1, p2) => UntypedForm::Disj(
            Box::new(trans_form(*p1, vars)),
            Box::new(trans_form(*p2, vars)),
        ),
        UntypedForm::Xor(p1, p2) => UntypedForm::Xor(
            Box::new(trans_form(*p1, vars)),
            Box::new(trans_form(*p2, vars)),
        ),
        UntypedForm::Impl(p1, p2) => UntypedForm::Impl(
            Box::new(trans_form(*p1, vars)),
            Box::new(trans_form(*p2, vars)),
        ),
        UntypedForm::BiImpl(p1, p2) => UntypedForm::BiImpl(
            Box::new(trans_form(*p1, vars)),
            Box::new(trans_form(*p2, vars)),
        ),
        UntypedForm::Forall(_, _) => todo!(),
        UntypedForm::Exist(_, _) => todo!(),
        UntypedForm::ForG(x, ags, p) => UntypedForm::ForG(
            meta::Agent(x),
            ags.into_iter().map(meta::Agent).collect(),
            Box::new(trans_form(*p, vars)),
        ),
        UntypedForm::K(ag, p) => UntypedForm::K(
            UntypedRef(meta::Agent(ag.0)),
            Box::new(trans_form(*p, vars)),
        ),
        UntypedForm::CK(ags, p) => UntypedForm::CK(
            ags.into_iter()
                .map(|ag| UntypedRef(meta::Agent(ag.0)))
                .collect(),
            Box::new(trans_form(*p, vars)),
        ),
        UntypedForm::DK(ags, p) => UntypedForm::DK(
            ags.into_iter()
                .map(|ag| UntypedRef(meta::Agent(ag.0)))
                .collect(),
            Box::new(trans_form(*p, vars)),
        ),
    }
}
