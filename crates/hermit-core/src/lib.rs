use flow::Formula;

mod flow;
mod network;

// /// A reference an agent.
// #[derive(Debug, Clone)]
// pub struct AgentRef<A> {
// name: A,
// }

// /// A `have` assertion.
// #[derive(Debug, Clone)]
// pub struct HaveInfo<A, V> {
// form: Formula<A, V>,
// }

// #[derive(Debug, Clone)]
// pub struct EnsureInfo<A, V> {
// form: Formula<A, V>,
// }

// #[derive(Debug, Clone)]
// pub struct ForgetInfo<L> {
// subject: L,
// dependencies: L,
// }

// #[derive(Debug, Clone)]
// pub struct FunctionInfo<A, L, V> {
// agent: AgentRef<A>,
// haves: Vec<HaveInfo<A, V>>,
// ensures: Vec<EnsureInfo<A, V>>,
// forgets: Vec<ForgetInfo<A, V>>,
// }

// #[derive(Debug, Clone)]
// pub struct Info {
// functions: Vec<FunctionInfo<A, V>>,
// }
