use std::collections::HashMap;

pub trait FlowAnalysis {
    type Id;

    fn flow(&self, name: &Self::Id) -> Vec<Self::Id>;
}

pub type IdAgents<I, A> = HashMap<I, A>;
pub type Formula<I, A> = epistemic::Formula<A, I>;

pub struct Hermit<I, A, F>
where
    F: FlowAnalysis<Id = I>,
{
    law: Vec<Formula<I, A>>,
    agents: IdAgents<I, A>,
    flow: F,
}

impl<I, A, F> Hermit<I, A, F>
where
    I: Clone,
    A: Clone,
    F: FlowAnalysis<Id = I>,
{
    pub fn new(law: Vec<Formula<I, A>>, agents: IdAgents<I, A>, flow: F) -> Self {
        Self { law, agents, flow }
    }

    pub fn analyse(&self, forms: impl IntoIterator<Item = Formula<I, A>>) -> bool
    where
        F: FlowAnalysis,
    {
        todo!()
    }
}
