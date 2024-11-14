use std::collections::BTreeMap;

/// A group of agents.
#[derive(Debug, Clone)]
pub struct Group<A> {
    /// The members of the group.
    pub members: Vec<A>,
}

/// A channel for semi-private group announcements.
#[derive(Debug, Clone)]
pub struct Channel<A> {
    /// The group of agents listening at the channel.
    pub listeners: Group<A>,
}

/// A network of agents and channels.
#[derive(Debug, Clone)]
pub struct Network<A, L> {
    /// The channels at each location.
    pub channels: BTreeMap<L, Channel<A>>,
}

/// A semi-private group announcement.
#[derive(Debug, Clone)]
pub struct Announcement<A, V> {
    /// The receivers of the announcement.
    pub target: Group<A>,
    /// The content of the announcement.
    pub val: V,
}

impl<A> Group<A> {
    /// Create a new group with the given `members`.
    pub fn new(members: Vec<A>) -> Self {
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
