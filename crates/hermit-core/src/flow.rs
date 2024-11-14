use std::collections::{BTreeSet, VecDeque};

/// The dissemination of data between locations.
pub trait Flow {
    /// The type of locations.
    type Location;

    /// Compute the locations to which the data of `loc` is _directly_ sent.
    fn forward(&self, loc: Self::Location) -> impl Iterator<Item = Self::Location>;

    /// Add another flow to this one.
    fn add<G>(self, other: G) -> Combined<Self, G>
    where
        Self: Sized,
        G: Flow<Location = Self::Location>,
    {
        Combined::new(self, other)
    }

    /// Subtract another flow from this one.
    fn sub<G>(self, other: G) -> Blocked<Self, G>
    where
        Self: Sized,
        G: Flow<Location = Self::Location>,
    {
        Blocked::new(self, other)
    }

    fn trans(self) -> Transitive<Self>
    where
        Self: Sized,
    {
        Transitive::new(self)
    }
}

/// The addition of two flows.
#[derive(Debug, Clone)]
pub struct Combined<F, G> {
    flow: F,
    glow: G,
}

impl<F, G> Combined<F, G> {
    /// Add `flow` and `glow`.
    pub fn new(flow: F, glow: G) -> Self {
        Self { flow, glow }
    }
}

impl<F, G> Flow for Combined<F, G>
where
    F: Flow,
    G: Flow<Location = F::Location>,
    F::Location: Copy,
{
    type Location = F::Location;

    fn forward(&self, loc: Self::Location) -> impl Iterator<Item = Self::Location> {
        self.flow.forward(loc).chain(self.glow.forward(loc))
    }
}

/// The subtraction of two flows.
#[derive(Debug, Clone)]
pub struct Blocked<F, G> {
    flow: F,
    blockage: G,
}

impl<F, G> Blocked<F, G> {
    /// Subtract `blockage` from `flow`.
    pub fn new(flow: F, blockage: G) -> Self {
        Self { flow, blockage }
    }
}

impl<F, G> Flow for Blocked<F, G>
where
    F: Flow,
    G: Flow<Location = F::Location>,
    F::Location: Ord + Copy,
{
    type Location = F::Location;

    fn forward(&self, loc: Self::Location) -> impl Iterator<Item = Self::Location> {
        let blocked: BTreeSet<_> = self.blockage.forward(loc).collect();
        self.flow
            .forward(loc)
            .filter(move |loc| !blocked.contains(loc))
    }
}

#[derive(Debug, Clone)]
pub struct Transitive<F> {
    flow: F,
}

impl<F> Transitive<F> {
    pub fn new(flow: F) -> Self {
        Self { flow }
    }
}

impl<F> Flow for Transitive<F>
where
    F: Flow,
    F::Location: Ord + Copy,
{
    type Location = F::Location;

    fn forward(&self, loc: Self::Location) -> impl Iterator<Item = Self::Location> {
        TransitiveIter::new(&self.flow, loc)
    }
}

pub struct TransitiveIter<'f, F>
where
    F: Flow,
{
    flow: &'f F,

    seen: BTreeSet<F::Location>,
    queue: VecDeque<F::Location>,
    iter: Box<dyn Iterator<Item = F::Location> + 'f>,
}

impl<'f, F> TransitiveIter<'f, F>
where
    F: Flow,
    F::Location: Ord + Copy,
{
    pub fn new(flow: &'f F, loc: F::Location) -> Self {
        let seen = BTreeSet::new();
        let queue = VecDeque::new();
        let iter = Box::new(flow.forward(loc));
        Self {
            flow,
            seen,
            queue,
            iter,
        }
    }
}

impl<'f, F> Iterator for TransitiveIter<'f, F>
where
    F: Flow,
    F::Location: Ord + Copy,
{
    type Item = F::Location;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(loc) = self.iter.next() {
            // skip if already seen.
            if !self.seen.insert(loc) {
                continue;
            }

            // queue up this location for recursion.
            self.queue.push_back(loc);

            return Some(loc);
        }

        // pop the next location that hasn't been recursed on yet.
        if let Some(loc) = self.queue.pop_front() {
            // update the current iterator and try again.
            self.iter = Box::new(self.flow.forward(loc));
            self.next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use maplit::*;

    use super::*;

    impl Flow for HashMap<usize, Vec<usize>> {
        type Location = usize;

        fn forward(&self, loc: Self::Location) -> impl Iterator<Item = Self::Location> {
            match self.get(&loc) {
                Some(locs) => Box::new(locs.iter().copied()),
                None => Box::new([].iter().copied()),
            }
        }
    }

    #[test]
    fn test_forward() {
        let map = hashmap! {
            0usize => vec![1usize],
            1 => vec![2],
            2 => vec![0, 1],
        };

        assert_eq!(map.forward(0).collect::<Vec<_>>(), vec![1usize]);
        assert_eq!(map.forward(1).collect::<Vec<_>>(), vec![2usize]);
        assert_eq!(map.forward(2).collect::<Vec<_>>(), vec![0usize, 1]);
    }

    #[test]
    fn test_trans() {
        let map = hashmap! {
            0usize => vec![1usize],
            1 => vec![2],
            2 => vec![0],
        }
        .trans();

        assert_eq!(map.forward(0).collect::<Vec<_>>(), vec![1usize, 2usize, 0]);

        let map = hashmap! {
            0usize => vec![1usize],
            1 => vec![2],
            2 => vec![0, 1, 2],
        }
        .trans();

        assert_eq!(map.forward(0).collect::<Vec<_>>(), vec![1usize, 2usize, 0]);
    }
}
