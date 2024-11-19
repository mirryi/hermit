use std::collections::{BTreeMap, BTreeSet};

use super::flow::{Flow, Transitive};

pub use epistemic::{Form, KnowStruct, Semantics};

/// A group of agents.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Group<A> {
    /// The members of the group.
    pub members: BTreeSet<A>,
}

/// A channel for semi-private group announcements.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Channel<A> {
    /// The group of agents listening at the channel.
    pub listeners: Group<A>,
}

/// A collection of channels.
#[derive(Debug, Clone)]
pub struct Network<A, L> {
    /// The channels at each location.
    pub channels: BTreeMap<L, Channel<A>>,
}

/// A semi-private group announcement.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Announcement<A, V> {
    /// The receivers of the announcement.
    pub target: Group<A>,
    /// The content of the announcement.
    pub val: V,
}

impl<A> Group<A> {
    /// Create a new group with the given `members`.
    pub fn new(members: BTreeSet<A>) -> Self {
        Self { members }
    }
}

impl<A> FromIterator<A> for Group<A>
where
    A: Ord,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let members = iter.into_iter().collect();
        Self { members }
    }
}

impl<A> Channel<A> {
    /// Create a new channel with the given `listeners`.
    pub fn new(listeners: Group<A>) -> Self {
        Self { listeners }
    }
}

impl<A> Channel<A>
where
    A: Clone,
{
    /// Create an announcement of `val` through the channel to all its listeners.
    pub fn announcement<V>(&self, val: V) -> Announcement<A, V> {
        Announcement::new(self.listeners.clone(), val)
    }
}

impl<A, L> Network<A, L>
where
    L: Eq + Ord,
{
    /// Create a new network of channels.
    pub fn new(channels: BTreeMap<L, Channel<A>>) -> Self {
        Self { channels }
    }

    /// Get the channel at the given location.
    pub fn channel(&self, loc: &L) -> Option<&Channel<A>> {
        self.channels.get(loc)
    }
}

impl<A, V> Announcement<A, V> {
    /// Create a new announcement of `val` to `target`.
    pub fn new(target: Group<A>, val: V) -> Self {
        Self { target, val }
    }
}

/// A summary of the flow of announcements on a [`Network`].
#[derive(Debug)]
pub struct NetworkFlow<A, F>
where
    F: Flow,
{
    /// The network.
    network: Network<A, F::Location>,
    /// The dissemination.
    flow: Transitive<F>,
}

impl<A, F> NetworkFlow<A, F>
where
    F: Flow,
{
    /// Create a new [`NetworkFlow`].
    pub fn new(network: Network<A, F::Location>, flow: F) -> Self {
        let flow = flow.trans();
        Self { network, flow }
    }
}

pub trait AnnouncementFlow {
    type Agent;
    type Location;

    /// Compute the _direct_ announcements of the data at `loc`.
    fn announcements(
        &self,
        loc: Self::Location,
    ) -> impl Iterator<Item = Announcement<Self::Agent, Self::Location>>;
}

impl<A, F> AnnouncementFlow for NetworkFlow<A, F>
where
    A: Clone,
    F: Flow,
    F::Location: Ord + Copy,
{
    type Agent = A;
    type Location = F::Location;

    fn announcements(
        &self,
        loc: F::Location,
    ) -> impl Iterator<
        Item = Announcement<
            <Self as AnnouncementFlow>::Agent,
            <Self as AnnouncementFlow>::Location,
        >,
    > {
        self.flow.forward(loc).filter_map(move |loc| {
            let chan = self.network.channel(&loc)?;
            Some(chan.announcement(loc))
        })
    }
}

/// The semantics of formulae with respect to an announcement flow and a knowledge structure.
pub struct AnnouncementFlowSat<F, K>
where
    F: AnnouncementFlow,
    K: KnowStruct<Agent = F::Agent, Prop = F::Location>,
{
    flow: F,
    know: K,
}

impl<F, K> AnnouncementFlowSat<F, K>
where
    F: AnnouncementFlow,
    K: KnowStruct<Agent = F::Agent, Prop = F::Location>,
{
    /// Create a [`NetworkFlowSat`].
    pub fn new(flow: F, know: K) -> Self {
        Self { flow, know }
    }
}

impl<F, K> Semantics for AnnouncementFlowSat<F, K>
where
    F: AnnouncementFlow,
    K: KnowStruct<Agent = F::Agent, Prop = F::Location>,
    F::Agent: Clone,
    F::Location: Ord + Copy,
{
    type Agent = F::Agent;
    type Prop = F::Location;

    fn sat(&self, form: Form<Self::Agent, Self::Prop>) -> bool {
        let anns: Vec<_> = form
            .vocab()
            .flat_map(|b| self.flow.announcements(*b))
            .collect();
        let form = anns.into_iter().fold(form, |form, ann| {
            let ags = ann.target.members.into_iter().collect();
            Form::GAw(ags, Box::new(Form::Prop(ann.val)), Box::new(form))
        });

        self.know.sat(form)
    }
}
