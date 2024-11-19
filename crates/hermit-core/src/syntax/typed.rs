use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use crate::semantics::{
    AnnouncementFlow, AnnouncementFlowSat, Channel, Flow, Group, KnowStruct, Network, NetworkFlow,
    Semantics,
};

pub type TypedForm<A, L> = epistemic::Form<A, L>;

#[derive(Debug, Clone)]
pub struct TypedMeta<A, L> {
    /// The owners of each location.
    ///
    /// This map defines the set of valid agents.
    pub owners: BTreeMap<L, BTreeSet<A>>,

    /// The `have` assertions.
    pub haves: Vec<TypedForm<A, L>>,
    /// The `ensures` assertions.
    pub ensures: Vec<TypedForm<A, L>>,
    /// The `forgets` assertions.
    pub forgets: Vec<TypedForget<L>>,
}

/// An `forget` assertion.
#[derive(Debug, Clone)]
pub struct TypedForget<L> {
    pub subject: L,
    pub dependencies: Vec<L>,
}

impl<L> TypedForget<L> {
    pub fn new(subject: L, dependencies: Vec<L>) -> Self {
        Self {
            subject,
            dependencies,
        }
    }
}

impl<A, L> TypedMeta<A, L> {
    pub fn new(
        owners: BTreeMap<L, BTreeSet<A>>,
        haves: Vec<TypedForm<A, L>>,
        ensures: Vec<TypedForm<A, L>>,
        forgets: Vec<TypedForget<L>>,
    ) -> Self {
        Self {
            owners,
            haves,
            ensures,
            forgets,
        }
    }
}

impl<A, L> TypedMeta<A, L>
where
    A: Ord,
{
    pub fn know_struct<'i, K>(&'i self) -> K
    where
        K: KnowStruct<Agent = &'i A, Prop = &'i L>,
    {
        let all_ags = self.owners.iter().flat_map(|(_, ags)| ags);

        let vocab: Vec<_> = self
            .haves
            .iter()
            .chain(self.ensures.iter())
            .flat_map(TypedForm::vocab)
            .collect();
        let law = TypedForm::Conj(
            self.haves
                .iter()
                .chain(self.ensures.iter())
                .map(Into::into)
                .collect(),
        );
        let obs = all_ags.map(|ag| (ag, vocab.clone())).collect();
        K::new(vocab, law, obs)
    }
}

impl<A, L> TypedMeta<A, L>
where
    A: Ord,
    L: Ord,
{
    pub fn network(&self) -> Network<&A, &L> {
        let channels = self
            .owners
            .iter()
            .map(|(loc, group)| (loc, Channel::new(Group::new(group.iter().collect()))))
            .collect();
        Network::new(channels)
    }

    pub fn announcement_flow<'i, F>(
        &'i self,
        flow: F,
    ) -> impl AnnouncementFlow<Agent = &'i A, Location = &'i L>
    where
        F: Flow<Location = &'i L>,
    {
        let blockage = self
            .forgets
            .iter()
            .map(|forgets| (&forgets.subject, forgets.dependencies.iter().collect()))
            .collect();

        let network = self.network();
        let flow = flow.sub(ForgetsFlow::new(blockage));
        NetworkFlow::new(network, flow)
    }

    pub fn semantics<'i, F, K>(&'i self, flow: F) -> impl Semantics<Agent = &'i A, Prop = &'i L>
    where
        F: Flow<Location = &'i L>,
        K: KnowStruct<Agent = &'i A, Prop = &'i L>,
    {
        let know: K = self.know_struct();
        let flow = self.announcement_flow(flow);
        AnnouncementFlowSat::new(flow, know)
    }
}

#[derive(Debug, Clone)]
struct ForgetsFlow<L>(BTreeMap<L, BTreeSet<L>>);

impl<L> ForgetsFlow<L> {
    pub fn new(map: BTreeMap<L, BTreeSet<L>>) -> Self {
        Self(map)
    }
}

impl<L> Flow for ForgetsFlow<L>
where
    L: Ord + Copy,
{
    type Location = L;

    fn forward(&self, loc: Self::Location) -> impl Iterator<Item = Self::Location> {
        let iter: Box<dyn Iterator<Item = _>> = match self.0.get(&loc) {
            Some(locs) => Box::new(locs.iter().copied()),
            None => Box::new(iter::empty()),
        };

        iter
    }
}
