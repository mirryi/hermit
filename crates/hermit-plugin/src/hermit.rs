// use flowistry::infoflow::{self, Direction, FlowResults};
// use rustc_borrowck::consumers::BodyWithBorrowckFacts;
// use rustc_hir::{BodyId, ItemKind};
// use rustc_middle::{
// mir::{Local, Location, Place},
// ty::TyCtxt,
// };
// use rustc_utils::{
// mir::{borrowck_facts, location_or_arg::LocationOrArg},
// PlaceExt,
// };

// use hermit_core::Flow;

// /// The flow analysis for a single function body.
// pub struct LocalFlowAnalysis<'tcx, 'f> {
// tcx: TyCtxt<'tcx>,
// results: FlowResults<'f, 'tcx>,
// }

// impl<'tcx, 'f> LocalFlowAnalysis<'tcx, 'f> {
// pub fn compute(
// tcx: TyCtxt<'tcx>,
// body_id: BodyId,
// body_with_facts: &BodyWithBorrowckFacts<'tcx>,
// ) -> Self {
// let results = infoflow::compute_flow(tcx, body_id, body_with_facts);
// Self { tcx, results }
// }
// }

// impl<'tcx, 'f> Flow for LocalFlowAnalysis<'tcx, 'f> {
// type Location = Location;
// type Value = Local;

// fn locations(&self, local: Self::Value) -> impl Iterator<Item = Self::Location> {
// let place = Place::make(local, &[], self.tcx);
// let targets = vec![vec![(place, LocationOrArg::Arg(local))]];
// let location_deps =
// infoflow::compute_dependencies(&self.results, targets, Direction::Forward).remove(0);
// }
// }

// /// The flow analysis for an entire crate.
// pub struct CrateFlowAnalysis<'tcx> {
// tcx: TyCtxt<'tcx>,
// body_ids: Vec<BodyId>,
// }

// impl<'tcx> CrateFlowAnalysis<'tcx> {
// pub fn compute(tcx: TyCtxt<'tcx>) -> Self {
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

// compute_dependencies(tcx, body_id, body_with_facts)
// }
// }

// impl<'tcx> Flow for CrateFlowAnalysis<'tcx> {
// type Location = ;
// type Value;

// fn locations(&self, value: Self::Value) -> impl Iterator<Item = Self::Location> {
// todo!()
// }
// }
