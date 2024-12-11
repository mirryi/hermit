use std::collections::BTreeMap;

use hermit_core::UntypedMeta;

use super::{Agent, Function, GlobalLocation, Meta};

impl Meta {
    pub fn untyped_meta_flow(&self) -> (UntypedMeta<Agent, GlobalLocation>, GlobalFlow) {
        // let mut owners = BTreeMap::new();
        // let mut haves = Vec::new();
        // let mut ensures = Vec::new();
        // let mut forgets = Vec::new();

        // for (id, fun) in &self.funs {
        // let Function {
        // agents,
        // haves,
        // ensures,
        // forgets,
        // flows,
        // } = fun;
        // }

        // (
        // UntypedMeta {
        // owners,
        // haves,
        // ensures,
        // forgets,
        // },
        // todo!(),
        // )

        todo!()
    }
}

pub struct LocalFLow {}

pub struct GlobalFlow {}
