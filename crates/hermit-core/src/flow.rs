use epistemic::KnowStruct;

use crate::network::{Announcement, Network};

/// The dissemination of values to locations.
pub trait ValueFlow {
    /// The type of locations.
    type Location: Ord;
    /// The type of values.
    type Value: Copy;

    /// Compute the locations to which `val` is sent.
    fn locations(&self, value: Self::Value) -> impl Iterator<Item = Self::Location>;
}

/// A summary of the flow of announcements on a [`Network`].
#[derive(Debug)]
pub struct NetworkFlow<A, F: ValueFlow> {
    /// The network.
    network: Network<A, F::Location>,
    /// The dissemination.
    flow: F,
}

impl<A, F: ValueFlow> NetworkFlow<A, F> {
    pub fn new(network: Network<A, F::Location>, flow: F) -> Self {
        Self { network, flow }
    }
}

impl<A, F: ValueFlow> NetworkFlow<A, F>
where
    A: Clone,
{
    /// Compute the announcements of `val`.
    pub fn announcements(
        &self,
        val: F::Value,
    ) -> impl Iterator<Item = Announcement<A, F::Value>> + '_ {
        let locs = self.flow.locations(val);
        locs.into_iter().filter_map(move |fdep| {
            let chan = self.network.channel(&fdep)?;
            Some(chan.announcement(val))
        })
    }
}

/// The shape of logical formulae.
pub type Formula<A, V> = epistemic::Form<A, V>;

pub struct NetworkFlowSat<A, F: ValueFlow, K: KnowStruct<A, F::Value>> {
    know: K,
    flow: NetworkFlow<A, F>,
}

impl<A, F: ValueFlow, K: KnowStruct<A, F::Value>> NetworkFlowSat<A, F, K> {
    /// Create a [`NetworkFlowSat`].
    pub fn new(know: K, flow: NetworkFlow<A, F>) -> Self {
        Self { know, flow }
    }
}

impl<A, F: ValueFlow, K: KnowStruct<A, F::Value>> NetworkFlowSat<A, F, K>
where
    A: Clone,
{
    pub fn sat_all(&self, forms: impl IntoIterator<Item = Formula<A, F::Value>>) -> bool {
        forms.into_iter().all(|form| self.sat(form))
    }

    pub fn sat(&self, form: Formula<A, F::Value>) -> bool {
        let anns = form
            .vocab()
            .flat_map(|b| self.flow.announcements(*b))
            .collect::<Vec<_>>();
        let form = anns.into_iter().fold(form, |form, ann| {
            Formula::GAw(
                ann.target.members,
                Box::new(Formula::Prop(ann.val)),
                Box::new(form),
            )
        });

        self.know.sat(form)
    }
}
