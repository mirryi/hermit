pub mod convert;

use std::collections::BTreeMap;

use rustc_middle::mir::Local;
use rustc_span::def_id::DefId;

use hermit_core::UntypedForm;

pub use hermit_syntax::{attribute::AgentMeta as AgentsAnn, lang::Agent};

/// The metadata of a program.
#[derive(Debug, Clone)]
pub struct Meta {
    pub funs: BTreeMap<FunctionId, Function>,
}

/// The identifier for a function.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct FunctionId(pub DefId);

/// The metadata of a function.
#[derive(Debug, Clone)]
pub struct Function {
    pub agents: Vec<AgentsAnn>,
    pub haves: Vec<HaveAnn>,
    pub ensures: Vec<EnsureAnn>,
    pub forgets: Vec<ForgetAnn>,

    /// The map of important locations to their dependent calls.
    pub flows: BTreeMap<LocalTarget, Vec<LocalTarget>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum LocalTarget {
    Local(FunctionLocation),
    Call(Call),
}

/// A location of interest inside a function body.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct FunctionLocation(pub Local);

/// The metadata of a call to another function that is tainted for a specific argument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Call {
    /// The called function.
    pub fun: FunctionId,
    /// The index of the tainted argument.
    pub idx: usize,
}

#[derive(Debug, Clone)]
pub struct HaveAnn {
    pub form: UntypedForm<Agent, LocalTarget>,
}

#[derive(Debug, Clone)]
pub struct EnsureAnn {
    pub form: UntypedForm<Agent, LocalTarget>,
}

#[derive(Debug, Clone)]
pub struct ForgetAnn {
    pub subject: LocalTarget,
    pub dependencies: Vec<LocalTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct GlobalTarget {
    pub body: FunctionId,
    pub local: LocalTarget,
}
