mod attr;
mod fun;

mod localflow;

use std::collections::BTreeMap;

use rustc_hir::{BodyId, ItemKind};
use rustc_middle::ty::TyCtxt;

use crate::meta;

use fun::FunCollector;

pub struct Collector<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> Collector<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self { tcx }
    }

    pub fn collect(&self) -> meta::Meta {
        let funs = self.collect_fns();
        meta::Meta { funs }
    }

    fn collect_fns(&self) -> BTreeMap<meta::FunctionId, meta::Function> {
        self.fn_body_ids()
            .map(|body_id| self.collect_fn(body_id))
            .collect()
    }

    fn collect_fn(&self, body_id: BodyId) -> (meta::FunctionId, meta::Function) {
        FunCollector::new(self.tcx, body_id).collect()
    }

    fn fn_body_ids(&self) -> impl Iterator<Item = BodyId> + 'tcx {
        let hir = self.tcx.hir();
        hir.items().filter_map(move |id| match hir.item(id).kind {
            ItemKind::Fn(_, _, body_id) => Some(body_id),
            _ => None,
        })
    }
}
