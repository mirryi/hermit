use std::collections::BTreeSet;

pub trait IterUniqueOrd: Iterator {
    fn unique_ord(self) -> UniqueOrd<Self>
    where
        Self: Sized,
        Self::Item: Clone + Eq + Ord,
    {
        UniqueOrd {
            iter: self.unique_ord_by(Box::new(|v| v.clone())),
        }
    }

    fn unique_ord_by<V, F>(self, f: F) -> UniqueOrdBy<Self, V, F>
    where
        Self: Sized,
        V: Eq + Ord,
        F: FnMut(&Self::Item) -> V,
    {
        UniqueOrdBy {
            iter: self,
            used: BTreeSet::new(),
            f,
        }
    }
}

impl<I> IterUniqueOrd for I where I: Iterator {}

pub struct UniqueOrd<I: Iterator> {
    iter: UniqueOrdBy<I, I::Item, Box<dyn FnMut(&I::Item) -> I::Item>>,
}

impl<I: Iterator> Iterator for UniqueOrd<I>
where
    I::Item: Ord,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter } = self;
        iter.next()
    }
}

pub struct UniqueOrdBy<I: Iterator, V, F> {
    iter: I,
    used: BTreeSet<V>,
    f: F,
}

impl<I: Iterator, V, F> Iterator for UniqueOrdBy<I, V, F>
where
    V: Ord,
    F: FnMut(&I::Item) -> V,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter, used, f } = self;
        iter.find(|v| used.insert(f(v)))
    }
}
