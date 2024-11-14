mod flow;
mod network;

use std::cmp::Ord;
use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use network::{Channel, Group, NetworkFlow};

pub use epistemic::{Form, KnowStruct, Semantics};
pub use flow::Flow;
pub use network::{AnnouncementFlow, AnnouncementFlowSat, Network};

/// A `have` assertion.
#[derive(Debug, Clone)]
pub struct Have<A, L> {
    form: Form<A, L>,
}

/// An `ensure` assertion.
#[derive(Debug, Clone)]
pub struct Ensure<A, L> {
    form: Form<A, L>,
}

/// An `forgets` assertion.
#[derive(Debug, Clone)]
pub struct Forgets<L> {
    subject: L,
    dependencies: Vec<L>,
}

/// The information for a function.
#[derive(Debug, Clone)]
pub struct Info<A, L> {
    /// The owners of each location.
    ///
    /// This map defines the set of valid agents.
    owners: BTreeMap<L, BTreeSet<A>>,

    /// The `have` assertions for this function.
    haves: Vec<Have<A, L>>,
    /// The `ensures` assertions for this function.
    ensures: Vec<Ensure<A, L>>,
    /// The `forgets` assertions for this function.
    forgets: Vec<Forgets<L>>,
}

impl<A, L> Have<A, L> {
    pub fn new(form: Form<A, L>) -> Self {
        Self { form }
    }

    pub fn vocab(&self) -> impl Iterator<Item = &L> {
        self.form.vocab()
    }
}

impl<A, L> Ensure<A, L> {
    pub fn new(form: Form<A, L>) -> Self {
        Self { form }
    }

    pub fn vocab(&self) -> impl Iterator<Item = &L> {
        self.form.vocab()
    }
}

impl<A, L> Info<A, L> {
    pub fn vocab(&self) -> impl Iterator<Item = &L> {
        self.haves
            .iter()
            .flat_map(Have::vocab)
            .chain(self.ensures.iter().flat_map(Ensure::vocab))
    }
}

impl<A, L> Info<A, L>
where
    A: Ord,
{
    pub fn know_struct<'i, K>(&'i self) -> K
    where
        K: KnowStruct<Agent = &'i A, Prop = &'i L>,
    {
        let vocab: Vec<_> = self.vocab().collect();
        let law = Form::Conj(
            self.haves
                .iter()
                .map(|have| &have.form)
                .map(Into::into)
                .collect(),
        );

        let ags: BTreeSet<_> = self.owners.iter().flat_map(|(_, ags)| ags).collect();
        let obs = ags.into_iter().map(|ag| (ag, vocab.clone())).collect();
        K::new(vocab, law, obs)
    }
}

impl<A, L> Info<A, L>
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
pub struct ForgetsFlow<L>(BTreeMap<L, BTreeSet<L>>);

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
