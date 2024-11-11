#[derive(Debug, Clone)]
pub enum Formula<A, P> {
    Top,
    Bot,
    Prop(P),
    Neg(Box<Self>),
    Conj(Vec<Self>),
    Disj(Vec<Self>),
    Xor(Vec<Self>),
    Impl(Box<Self>, Box<Self>),
    Equiv(Box<Self>, Box<Self>),
    Forall(Vec<P>, Box<Self>),
    Exist(Vec<P>, Box<Self>),
    K(A, Box<Self>),
    CK(Vec<A>, Box<Self>),
    DK(Vec<A>, Box<Self>),
    CKw(Vec<A>, Box<Self>),
    DKw(Vec<A>, Box<Self>),
    PA(Box<Self>, Box<Self>),
    PAw(Box<Self>, Box<Self>),
    GA(Vec<A>, Box<Self>, Box<Self>),
    GAw(Vec<A>, Box<Self>, Box<Self>),
}

pub trait HasSemantics<A, P> {
    fn sat(&self, form: Formula<A, P>) -> bool;
}
