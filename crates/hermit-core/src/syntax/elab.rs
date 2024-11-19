use immutable_list::List;
use iter_unique_ord::IterUniqueOrd;

use super::{TypedForm, TypedMeta, UntypedForm, UntypedMeta, UntypedRef};

/// An error that arises during type-checking.
#[derive(Debug, Clone)]
pub enum ElabError<A> {
    /// The use of an unbound agent reference.
    FreeAgent(AgentContext<A>, A),
}

impl<A, L> UntypedMeta<A, L>
where
    A: Ord + Clone,
    L: Ord + Clone,
{
    pub fn elab(self) -> Result<TypedMeta<A, L>, ElabError<A>> {
        let UntypedMeta {
            owners,
            haves,
            ensures,
            forgets,
        } = self;

        let ctx: AgentContext<_> = owners
            .iter()
            .flat_map(|(_, ags)| ags)
            .unique_ord()
            .cloned()
            .map(|ag| (ag.clone(), ag))
            .collect();

        // elab `have` and `ensure` assertions
        let haves = haves
            .into_iter()
            .map(|p| p.elab(ctx.clone()))
            .collect::<Result<_, _>>()?;
        let ensures: Vec<_> = ensures
            .into_iter()
            .map(|p| p.elab(ctx.clone()))
            .collect::<Result<_, _>>()?;

        // elab `forget` assertions.
        let forgets = forgets.into_iter().map(|f| f.elab()).collect();

        Ok(TypedMeta::new(owners, haves, ensures, forgets))
    }
}

impl<A, L> UntypedForm<A, L>
where
    A: Ord + Clone,
    L: Clone,
{
    /// Elaborate into the equivalent [`TypedForm`].
    pub fn elab(self, atx: AgentContext<A>) -> Result<TypedForm<A, L>, ElabError<A>> {
        let form = match self {
            UntypedForm::Top => TypedForm::Top,
            UntypedForm::Bot => TypedForm::Bot,
            UntypedForm::Prop(b) => TypedForm::Prop(b),
            UntypedForm::Neg(p) => TypedForm::Neg(Box::new(p.elab(atx)?)),

            UntypedForm::Conj(p1, p2) => {
                TypedForm::Conj(vec![p1.elab(atx.clone())?, p2.elab(atx)?])
            }
            UntypedForm::Disj(p1, p2) => {
                TypedForm::Disj(vec![p1.elab(atx.clone())?, p2.elab(atx)?])
            }
            UntypedForm::Xor(p1, p2) => TypedForm::Xor(vec![p1.elab(atx.clone())?, p2.elab(atx)?]),

            UntypedForm::Impl(p1, p2) => {
                TypedForm::Impl(Box::new(p1.elab(atx.clone())?), Box::new(p2.elab(atx)?))
            }
            UntypedForm::BiImpl(p1, p2) => {
                TypedForm::Equiv(Box::new(p1.elab(atx.clone())?), Box::new(p2.elab(atx)?))
            }

            UntypedForm::Forall(ps, p) => TypedForm::Forall(ps, Box::new(p.elab(atx.clone())?)),
            UntypedForm::Exist(ps, p) => TypedForm::Exist(ps, Box::new(p.elab(atx.clone())?)),

            UntypedForm::ForG(rf, set, p) => {
                let ps = if set.is_empty() {
                    atx.iter()
                        .map(|(_, ag)| ag)
                        .unique_ord()
                        .map(|ag| p.clone().elab(atx.extend(rf.clone(), ag.clone())))
                        .collect::<Result<_, _>>()?
                } else {
                    set.into_iter()
                        .map(|ag| p.clone().elab(atx.extend(rf.clone(), ag)))
                        .collect::<Result<_, _>>()?
                };
                TypedForm::Conj(ps)
            }

            UntypedForm::K(rf, p) => TypedForm::K(rf.elab(atx.clone())?, Box::new(p.elab(atx)?)),
            UntypedForm::CK(rfs, p) => TypedForm::CK(
                rfs.into_iter()
                    .map(|rf| rf.elab(atx.clone()))
                    .collect::<Result<_, _>>()?,
                Box::new(p.elab(atx)?),
            ),
            UntypedForm::DK(rfs, p) => TypedForm::DK(
                rfs.into_iter()
                    .map(|rf| rf.elab(atx.clone()))
                    .collect::<Result<_, _>>()?,
                Box::new(p.elab(atx)?),
            ),
        };

        Ok(form)
    }
}

impl<A> UntypedRef<A>
where
    A: Ord + Clone,
{
    pub fn elab(self, atx: AgentContext<A>) -> Result<A, ElabError<A>> {
        let Self(rf) = self;
        atx.lookup(rf).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct AgentContext<A> {
    inner: List<(A, A)>,
}

impl<A> AgentContext<A> {
    pub fn empty() -> Self {
        Self { inner: List::new() }
    }

    pub fn extend(&self, rf: A, ag: A) -> Self {
        let Self { inner } = self;

        Self {
            inner: inner.cons((rf, ag)),
        }
    }

    pub fn list(&self) -> &List<(A, A)> {
        &self.inner
    }

    pub fn to_list(self) -> List<(A, A)> {
        self.inner
    }

    pub fn iter(&self) -> immutable_list::Iter<'_, (A, A)> {
        self.list().iter()
    }
}

impl<A> AgentContext<A>
where
    A: Eq + Clone,
{
    pub fn lookup(&self, rf: A) -> Result<&A, ElabError<A>> {
        self.inner
            .iter()
            .find_map(|(r, a)| if *r == rf { Some(a) } else { None })
            .ok_or_else(|| ElabError::FreeAgent(self.clone(), rf))
    }
}

impl<A> Default for AgentContext<A> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<'c, A> IntoIterator for &'c AgentContext<A> {
    type Item = &'c (A, A);
    type IntoIter = immutable_list::Iter<'c, (A, A)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A> FromIterator<(A, A)> for AgentContext<A> {
    fn from_iter<T: IntoIterator<Item = (A, A)>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::empty(), |ctx, (rf, ag)| ctx.extend(rf, ag))
    }
}
