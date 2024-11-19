use std::collections::{BTreeMap, BTreeSet};

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
