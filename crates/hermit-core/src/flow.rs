use crate::network::{Announcement, Network};

/// The dissemination of values to locations.
pub trait Flow {
    type Location;
    type Value;

    /// Compute the locations to which `val` is sent.
    fn locations(&self, value: Self::Value) -> impl Iterator<Item = Self::Location>;
}

/// A summary of the flow of announcements on a [`Network`].
#[derive(Debug)]
pub struct AnnouncementFlow<A, F: Flow> {
    /// The network.
    network: Network<F::Location, A>,
    /// The dissemination.
    flow: F,
}

impl<A, F: Flow> AnnouncementFlow<A, F>
where
    A: Clone,
    F::Location: Eq + Ord,
    F::Value: Copy,
{
    pub fn new(network: Network<F::Location, A>, flow: F) -> Self {
        Self { network, flow }
    }

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
