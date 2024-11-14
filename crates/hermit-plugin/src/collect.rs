use rustc_hir::ItemKind;
use rustc_middle::ty::TyCtxt;
use rustc_utils::mir::borrowck_facts;

#[derive(Debug, Clone)]
pub struct AgentInfo {
    name: String,
}

#[derive(Debug, Clone)]
pub struct AgentRef {
    name: String,
}

#[derive(Debug, Clone)]
pub struct HaveInfo {}

#[derive(Debug, Clone)]
pub struct EnsureInfo {}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    agent: AgentRef,
    haves: Vec<HaveInfo>,
    ensures: Vec<EnsureInfo>,
}

#[derive(Debug, Clone)]
pub struct Info {
    functions: Vec<FunctionInfo>,
}

// pub fn collect<'tcx>(tcx: TyCtxt<'tcx>) -> Info {
// let hir = tcx.hir();

// // collect the body ids.
// let body_ids = hir
// .items()
// .filter_map(|id| match hir.item(id).kind {
// ItemKind::Fn(_, _, body_id) => Some(body_id),
// _ => None,
// })
// .collect();

// let def_id = hir.body_owner_def_id(body_id);
// let body_with_facts = borrowck_facts::get_body_with_borrowck_facts(tcx, def_id);

// compute_dependencies(tcx, body_id, body_with_facts);

// todo!()
// }
