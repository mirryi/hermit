use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use rustc_ast::Attribute;
use rustc_borrowck::consumers::BodyWithBorrowckFacts;
use rustc_hir::BodyId;
use rustc_middle::{
    hir::map::Map as HirMap,
    mir::{
        Body, Local, Statement, StatementKind, Terminator, TerminatorKind, VarDebugInfo,
        VarDebugInfoContents,
    },
    ty::TyCtxt,
};
use rustc_span::def_id::LocalDefId;
use rustc_utils::mir::{borrowck_facts, location_or_arg::LocationOrArg};

use either::Either;
use itertools::Itertools;

use crate::meta;

use super::attr::{self, AttrCollector};
use super::localflow::{BodyFlow, DirectBodyFlow, FlowSet};

pub struct FunCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    body_id: BodyId,
    body_with_facts: &'tcx BodyWithBorrowckFacts<'tcx>,
}

impl<'tcx> FunCollector<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, body_id: BodyId) -> Self {
        let body_with_facts =
            borrowck_facts::get_body_with_borrowck_facts(tcx, tcx.hir().body_owner_def_id(body_id));
        Self {
            tcx,
            body_id,
            body_with_facts,
        }
    }

    fn hir(&self) -> HirMap<'tcx> {
        self.tcx.hir()
    }

    fn def_id(&self) -> LocalDefId {
        self.hir().body_owner_def_id(self.body_id)
    }

    fn body(&self) -> &Body<'tcx> {
        &self.body_with_facts.body
    }

    fn attrs(&self) -> &'tcx [Attribute] {
        self.tcx.get_attrs_unchecked(self.def_id().into())
    }

    pub fn collect(&self) -> (meta::FunctionId, meta::Function) {
        // collect locations for all arguments and variables mentioned in the attributes.
        let attrs = self.collect_attrs();
        let attrs_vars: BTreeSet<_> = attrs
            .iter()
            .flat_map(|attr| attr.variables())
            .map(|ident| &ident.0.value)
            .collect();
        let locs = self.arg_and_these_locations(|name| attrs_vars.contains(name));

        // compute the forward dependencies for each location.
        let flows = self.collect_flows(locs.into_iter().map(|(_, loc)| loc));

        println!("{:?}", self.body_id);
        println!("{:#?}", self.body());
        println!("{:#?}", flows);

        // process the attributes.
        let agents = Vec::new();
        let haves = Vec::new();
        let ensures = Vec::new();
        let forgets = Vec::new();
        (
            meta::FunctionId(self.def_id().into()),
            meta::Function {
                agents,
                haves,
                ensures,
                forgets,
                flows,
            },
        )
    }

    // Collect the attributes.
    fn collect_attrs(&self) -> Vec<attr::AttrInfo> {
        self.attrs()
            .into_iter()
            .map(AttrCollector::new)
            .filter_map(|col| col.collect())
            .collect()
    }

    /// Collect the flows of `locs`.
    fn collect_flows(
        &self,
        locs: impl IntoIterator<Item = meta::FunctionLocation>,
    ) -> BTreeMap<meta::Target, Vec<meta::Target>> {
        // recursively compute the direct flow from `locs`.
        let mut direct_flow = DirectBodyFlow::new(self.tcx, self.body_id);

        let mut computed = BTreeSet::new();
        let mut locals: Vec<_> = locs.into_iter().map(|loc| loc.0).collect();
        while !locals.is_empty() {
            let deps = direct_flow.compute(locals);

            locals = deps
                .iter()
                .flat_map(|(_, set)| set.assigned_locals())
                .filter(|local| !computed.contains(local))
                .collect();
            computed.extend(deps.keys().copied());
        }

        direct_flow
            .flows()
            .into_iter()
            .flat_map(|(local, deps)| self.targets_of_deps(local, deps))
            .fold(BTreeMap::new(), |mut acc, (t1, t2)| {
                match acc.entry(t1) {
                    Entry::Vacant(ent) => {
                        ent.insert(vec![t2]);
                    }
                    Entry::Occupied(mut ent) => {
                        ent.get_mut().push(t2);
                    }
                };
                acc
            })
    }

    /// Extract the set of calls from `deps`.
    fn targets_of_deps(
        &self,
        local: Local,
        deps: FlowSet<'tcx, '_>,
    ) -> Vec<(meta::Target, meta::Target)> {
        deps.iter()
            .filter_map(|loc_arg| match loc_arg {
                LocationOrArg::Arg(_) => None,
                LocationOrArg::Location(loc) => match self.body().stmt_at(*loc) {
                    Either::Left(Statement { kind, .. }) => match kind {
                        StatementKind::Assign(assign) => Some(vec![(
                            meta::Target::Local(meta::FunctionLocation(local)),
                            meta::Target::Local(meta::FunctionLocation(assign.0.local)),
                        )]),
                        _ => None,
                    },
                    Either::Right(Terminator { kind, .. }) => match kind {
                        TerminatorKind::Call {
                            func,
                            args,
                            destination,
                            ..
                        } => {
                            // handle only const function definitions.
                            let (id, _) = func.const_fn_def()?;

                            // for each argument in the taint set, map to a pair of dependencies.
                            let calls = args
                                .into_iter()
                                .enumerate()
                                .filter_map(|(i, arg)| arg.place().map(|place| (i, place.local)))
                                .filter(|(_, arg)| local == *arg)
                                .flat_map(move |(i, _)| {
                                    [
                                        (
                                            meta::Target::Local(meta::FunctionLocation(local)),
                                            meta::Target::Call(meta::Call {
                                                fun: meta::FunctionId(id),
                                                idx: i,
                                            }),
                                        ),
                                        (
                                            meta::Target::Call(meta::Call {
                                                fun: meta::FunctionId(id),
                                                idx: i,
                                            }),
                                            meta::Target::Local(meta::FunctionLocation(
                                                destination.local,
                                            )),
                                        ),
                                    ]
                                })
                                .collect::<Vec<_>>();

                            Some(calls)
                        }
                        _ => None,
                    },
                },
            })
            .flatten()
            .collect()
    }

    fn arg_and_these_locations(
        &self,
        include: impl Fn(&String) -> bool,
    ) -> BTreeMap<String, meta::FunctionLocation> {
        self.var_locations()
            .into_iter()
            .filter(|(name, loc)| self.is_arg_location(loc) || include(name))
            .collect()
    }

    /// Map the first occurrence of each source variable to the corresponding local.
    fn var_locations(&self) -> BTreeMap<String, meta::FunctionLocation> {
        self.body()
            .var_debug_info
            .iter()
            .filter_map(|VarDebugInfo { name, value, .. }| match value {
                VarDebugInfoContents::Place(place) => {
                    Some((name.as_str(), meta::FunctionLocation(place.local)))
                }
                VarDebugInfoContents::Const(_) => None,
            })
            .unique_by(|(name, _)| *name)
            .map(|(name, loc)| (name.to_string(), loc))
            .collect()
    }

    fn is_arg_location(&self, loc: &meta::FunctionLocation) -> bool {
        loc.0.as_usize() <= self.body().arg_count
    }
}
