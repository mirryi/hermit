use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use iter_tree::Tree;

use super::typed::TypedForget;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UntypedForm<A, L> {
    Top,
    Bot,
    Prop(L),
    Neg(Box<Self>),
    Conj(Box<Self>, Box<Self>),
    Disj(Box<Self>, Box<Self>),
    Xor(Box<Self>, Box<Self>),
    Impl(Box<Self>, Box<Self>),
    BiImpl(Box<Self>, Box<Self>),
    Forall(Vec<L>, Box<Self>),
    Exist(Vec<L>, Box<Self>),
    ForG(A, Vec<A>, Box<Self>),
    K(UntypedRef<A>, Box<Self>),
    CK(Vec<UntypedRef<A>>, Box<Self>),
    DK(Vec<UntypedRef<A>>, Box<Self>),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UntypedRef<A>(pub A);

/// The information for a function.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UntypedMeta<A, L>
where
    A: Ord,
    L: Ord,
{
    /// The owners of each location.
    ///
    /// This map defines the set of valid agents.
    pub owners: BTreeMap<L, BTreeSet<A>>,

    /// The `have` assertions.
    pub haves: Vec<UntypedForm<A, L>>,
    /// The `ensures` assertions.
    pub ensures: Vec<UntypedForm<A, L>>,
    /// The `forgets` assertions.
    pub forgets: Vec<UntypedForget<L>>,
}

/// An `forget` assertion.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UntypedForget<L> {
    pub subject: L,
    pub dependencies: Vec<L>,
}

impl<A, L> UntypedMeta<A, L>
where
    A: Ord,
    L: Ord,
{
    pub fn new(
        owners: BTreeMap<L, BTreeSet<A>>,
        haves: Vec<UntypedForm<A, L>>,
        ensures: Vec<UntypedForm<A, L>>,
        forgets: Vec<UntypedForget<L>>,
    ) -> Self {
        Self {
            owners,
            haves,
            ensures,
            forgets,
        }
    }
}

impl<L> UntypedForget<L> {
    pub fn new(subject: L, dependencies: Vec<L>) -> Self {
        Self {
            subject,
            dependencies,
        }
    }

    pub fn elab(self) -> TypedForget<L> {
        let Self {
            subject,
            dependencies,
        } = self;
        TypedForget::new(subject, dependencies)
    }
}

impl<A, L> Default for UntypedForm<A, L> {
    fn default() -> Self {
        Self::Top
    }
}

impl<A, L> UntypedForm<A, L> {
    /// Iterate over the propositions.
    pub fn vocab<'a>(&'a self) -> Box<dyn Iterator<Item = &'a L> + 'a> {
        match self.vocab_tree() {
            Some(tree) => Box::new(tree.into_iter()),
            None => Box::new(iter::empty()),
        }
    }

    fn vocab_tree(&self) -> Option<Tree<&L>> {
        match self {
            UntypedForm::Top | UntypedForm::Bot => None,
            UntypedForm::Prop(b) => Some(Tree::Leaf(b)),
            UntypedForm::Neg(p)
            | UntypedForm::Forall(_, p)
            | UntypedForm::Exist(_, p)
            | UntypedForm::ForG(_, _, p)
            | UntypedForm::K(_, p)
            | UntypedForm::CK(_, p)
            | UntypedForm::DK(_, p) => p.vocab_tree().map(|n| Tree::Node(vec![n])),
            UntypedForm::Conj(p1, p2)
            | UntypedForm::Disj(p1, p2)
            | UntypedForm::Xor(p1, p2)
            | UntypedForm::Impl(p1, p2)
            | UntypedForm::BiImpl(p1, p2) => {
                let n1 = p1.vocab_tree()?;
                let n2 = p2.vocab_tree()?;
                Some(Tree::Node(vec![n1, n2]))
            }
        }
    }
}
