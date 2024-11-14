mod flow;
mod network;

use std::cmp::Ord;
use std::collections::BTreeMap;

use epistemic::KnowStruct;
use flow::{Formula, NetworkFlow, NetworkFlowSat, ValueFlow};
use network::{Channel, Group, Network};

/// A `have` assertion.
#[derive(Debug, Clone)]
pub struct Have<A, V> {
    form: Formula<A, V>,
}

/// An `ensure` assertion.
#[derive(Debug, Clone)]
pub struct Ensure<A, V> {
    form: Formula<A, V>,
}

/// An `forgets` assertion.
#[derive(Debug, Clone)]
pub struct Forgets<L> {
    subject: L,
    dependencies: L,
}

/// The information for a function.
#[derive(Debug, Clone)]
pub struct Info<A, L, V> {
    /// The `have` assertions for this function.
    haves: Vec<Have<A, V>>,
    /// The `ensures` assertions for this function.
    ensures: Vec<Ensure<A, V>>,
    /// The `forgets` assertions for this function.
    forgets: Vec<Forgets<L>>,

    /// The owners of each location.
    owners: BTreeMap<L, Vec<A>>,
}

impl<A, L, V> Info<A, L, V> {
    pub fn know_struct<'a, K>(&'a self) -> K
    where
        K: KnowStruct<&'a A, &'a V>,
    {
        let vocab = self.vocab();
        let law = todo!();
        let obs = todo!();
        K::new(vocab, law, obs)
    }

    fn vocab(&self) -> Vec<&V> {
        todo!()
    }
}

impl<A, L, V> Info<A, L, V>
where
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

    pub fn network_flow<'a, F>(&'a self, flow: F) -> NetworkFlow<&'a A, F>
    where
        F: ValueFlow<Location = &'a L, Value = &'a V>,
    {
        let network = self.network();
        NetworkFlow::new(network, flow)
    }

    pub fn network_flow_sat<'a, F, K>(&'a self, flow: F) -> NetworkFlowSat<&'a A, F, K>
    where
        F: ValueFlow<Location = &'a L, Value = &'a V>,
        K: KnowStruct<&'a A, &'a V>,
    {
        let know = self.know_struct();
        let flow = self.network_flow(flow);
        NetworkFlowSat::new(know, flow)
    }
}
