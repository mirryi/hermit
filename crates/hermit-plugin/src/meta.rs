// use hermit_core::UntypedForm;

use hermit_core::UntypedForm;

pub use hermit_syntax::{
    attribute::{
        AgentMeta as Agent, // EnsureMeta as Ensure, ForgetMeta as Forget, HaveMeta as Have,
    },
    lang::Agent as AgentRef,
};
use rustc_middle::mir::Place;

#[derive(Debug, Clone)]
pub struct Location<'tcx> {
    place: Place<'tcx>,
}

#[derive(Debug, Clone)]
pub struct Have<'tcx> {
    pub form: UntypedForm<AgentRef, Location<'tcx>>,
}

#[derive(Debug, Clone)]
pub struct Ensure<'tcx> {
    pub form: UntypedForm<AgentRef, Location<'tcx>>,
}

#[derive(Debug, Clone)]
pub struct Forget<'tcx> {
    pub subject: Location<'tcx>,
    pub dependencies: Location<'tcx>,
}

#[derive(Debug, Clone)]
pub struct FunctionName {}

#[derive(Debug, Clone)]
pub struct Call<'tcx> {
    pub name: FunctionName,
    pub args: Vec<Location<'tcx>>,
}

#[derive(Debug, Clone)]
pub struct Function<'tcx> {
    pub agents: Vec<Agent>,
    pub haves: Vec<Have<'tcx>>,
    pub ensures: Vec<Ensure<'tcx>>,
    pub forgets: Vec<Forget<'tcx>>,

    pub calls: Vec<Call<'tcx>>,
}

#[derive(Debug, Clone)]
pub struct Meta<'tcx> {
    pub fns: Vec<Function<'tcx>>,
}
