mod flow;
mod network;

use epistemic::KnowStruct;
use flow::AnnouncementFlow;

pub use flow::Flow;

pub type Formula<A, V> = epistemic::Form<A, V>;

pub struct Hermit<A, F: Flow, K: KnowStruct<A, F::Value>> {
    know: K,
    flow: AnnouncementFlow<A, F>,
}

impl<A, F: Flow, K: KnowStruct<A, F::Value>> Hermit<A, F, K>
where
    A: Clone,
    F::Location: Eq + Ord,
    F::Value: Copy,
{
    pub fn new(
        agents: Vec<A>,
        interest: Vec<F::Value>,
        laws: Vec<Formula<A, F::Value>>,
        flow: AnnouncementFlow<A, F>,
    ) -> Self {
        let law = Formula::Conj(laws);
        let obs = agents.into_iter().map(|a| (a, interest.clone())).collect();
        let know = K::new(interest, law, obs);
        Self { know, flow }
    }

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
