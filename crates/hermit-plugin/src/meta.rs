pub use hermit_syntax::attribute::{
    AgentMeta as Agent, EnsureMeta as Ensure, ForgetMeta as Forget, HaveMeta as Have,
};

#[derive(Debug, Clone)]
pub struct Location {}

#[derive(Debug, Clone)]
pub struct FunctionName {}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: FunctionName,
    pub args: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub agents: Vec<Agent>,
    pub haves: Vec<Have>,
    pub ensures: Vec<Ensure>,
    pub forgets: Vec<Forget>,

    pub calls: Vec<Call>,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub fns: Vec<Function>,
}
