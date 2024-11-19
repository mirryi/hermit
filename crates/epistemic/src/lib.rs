use std::iter;

use iter_tree::Tree;

pub trait Semantics {
    type Agent;
    type Prop;

    fn sat(&self, form: Form<Self::Agent, Self::Prop>) -> bool;
}

pub trait KnowStruct: Semantics {
    fn new(
        vocab: Vec<Self::Prop>,
        law: Form<Self::Agent, Self::Prop>,
        obs: Vec<(Self::Agent, Vec<Self::Prop>)>,
    ) -> Self;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Form<A, P> {
    Top,
    Bot,
    Prop(P),
    Neg(Box<Self>),
    Conj(Vec<Self>),
    Disj(Vec<Self>),
    Xor(Vec<Self>),
    Impl(Box<Self>, Box<Self>),
    Equiv(Box<Self>, Box<Self>),
    Forall(Vec<P>, Box<Self>),
    Exist(Vec<P>, Box<Self>),
    K(A, Box<Self>),
    CK(Vec<A>, Box<Self>),
    DK(Vec<A>, Box<Self>),
    CKw(Vec<A>, Box<Self>),
    DKw(Vec<A>, Box<Self>),
    PA(Box<Self>, Box<Self>),
    PAw(Box<Self>, Box<Self>),
    GA(Vec<A>, Box<Self>, Box<Self>),
    GAw(Vec<A>, Box<Self>, Box<Self>),
}

impl<A, P> Form<A, P> {
    /// Iterate over the propositions.
    pub fn vocab<'a>(&'a self) -> Box<dyn Iterator<Item = &'a P> + 'a> {
        match self.vocab_tree() {
            Some(tree) => Box::new(tree.into_iter()),
            None => Box::new(iter::empty()),
        }
    }

    fn vocab_tree(&self) -> Option<Tree<&P>> {
        match self {
            Form::Top | Form::Bot => None,
            Form::Prop(b) => Some(Tree::Leaf(b)),
            Form::Conj(ps) | Form::Disj(ps) | Form::Xor(ps) => {
                let ns = ps.iter().filter_map(Self::vocab_tree).collect();
                Some(Tree::Node(ns))
            }
            Form::Neg(p)
            | Form::Forall(_, p)
            | Form::Exist(_, p)
            | Form::K(_, p)
            | Form::CK(_, p)
            | Form::DK(_, p)
            | Form::CKw(_, p)
            | Form::DKw(_, p) => p.vocab_tree().map(|n| Tree::Node(vec![n])),
            Form::Impl(p1, p2)
            | Form::Equiv(p1, p2)
            | Form::PA(p1, p2)
            | Form::PAw(p1, p2)
            | Form::GA(_, p1, p2)
            | Form::GAw(_, p1, p2) => {
                let n1 = p1.vocab_tree()?;
                let n2 = p2.vocab_tree()?;
                Some(Tree::Node(vec![n1, n2]))
            }
        }
    }
}

impl<'p, A, P> From<&'p Form<A, P>> for Form<&'p A, &'p P> {
    fn from(form: &'p Form<A, P>) -> Self {
        match form {
            Form::Top => Form::Top,
            Form::Bot => Form::Bot,
            Form::Prop(b) => Form::Prop(b),
            Form::Neg(p) => Form::Neg(Box::new(p.as_ref().into())),
            Form::Conj(ps) => Form::Conj(ps.iter().map(Into::into).collect()),
            Form::Disj(ps) => Form::Conj(ps.iter().map(Into::into).collect()),
            Form::Xor(ps) => Form::Conj(ps.iter().map(Into::into).collect()),
            Form::Impl(p1, p2) => {
                Form::Impl(Box::new(p1.as_ref().into()), Box::new(p2.as_ref().into()))
            }
            Form::Equiv(p1, p2) => {
                Form::Equiv(Box::new(p1.as_ref().into()), Box::new(p2.as_ref().into()))
            }
            Form::Forall(xs, p) => Form::Forall(xs.iter().collect(), Box::new(p.as_ref().into())),
            Form::Exist(xs, p) => Form::Exist(xs.iter().collect(), Box::new(p.as_ref().into())),
            Form::K(ag, p) => Form::K(ag, Box::new(p.as_ref().into())),
            Form::CK(ags, p) => Form::CK(ags.iter().collect(), Box::new(p.as_ref().into())),
            Form::DK(ags, p) => Form::DK(ags.iter().collect(), Box::new(p.as_ref().into())),
            Form::CKw(ags, p) => Form::CKw(ags.iter().collect(), Box::new(p.as_ref().into())),
            Form::DKw(ags, p) => Form::DKw(ags.iter().collect(), Box::new(p.as_ref().into())),
            Form::PA(p1, p2) => {
                Form::PA(Box::new(p1.as_ref().into()), Box::new(p2.as_ref().into()))
            }
            Form::PAw(p1, p2) => {
                Form::PAw(Box::new(p1.as_ref().into()), Box::new(p2.as_ref().into()))
            }
            Form::GA(ags, p1, p2) => Form::GA(
                ags.iter().collect(),
                Box::new(p1.as_ref().into()),
                Box::new(p2.as_ref().into()),
            ),
            Form::GAw(ags, p1, p2) => Form::GAw(
                ags.iter().collect(),
                Box::new(p1.as_ref().into()),
                Box::new(p2.as_ref().into()),
            ),
        }
    }
}
