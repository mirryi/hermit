use std::iter;

use iter_tree::Tree;

pub trait Semantics<A, P> {
    fn sat(&self, form: Form<A, P>) -> bool;
}

pub trait KnowStruct<A, P>: Semantics<A, P> {
    fn new(vocab: Vec<P>, law: Form<A, P>, obs: Vec<(A, Vec<P>)>) -> Self;
}

#[derive(Debug, Clone)]
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
