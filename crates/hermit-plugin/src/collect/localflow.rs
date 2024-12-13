#![allow(dead_code)]

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    ops::{Deref, Sub},
};

use rustc_borrowck::consumers::BodyWithBorrowckFacts;
use rustc_hir::BodyId;
use rustc_middle::{
    mir::{Body, Local, Location, Place, Statement, StatementKind, Terminator, TerminatorKind},
    ty::TyCtxt,
};
use rustc_utils::{
    mir::{
        borrowck_facts,
        location_or_arg::{index::LocationOrArgSet, LocationOrArg},
    },
    PlaceExt,
};

use derivative::Derivative;
use either::Either;
use flowistry::infoflow::{Direction, FlowResults};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FlowSet<'tcx, 's> {
    #[derivative(Debug = "ignore")]
    body: &'tcx Body<'tcx>,

    set: &'s HashSet<LocationOrArg>,
}

impl<'tcx, 's> FlowSet<'tcx, 's> {
    fn new(body: &'tcx Body<'tcx>, set: &'s HashSet<LocationOrArg>) -> Self {
        Self { body, set }
    }

    pub fn assigned_locals(&self) -> BTreeSet<Local> {
        self.set
            .iter()
            .filter_map(|loc_arg| match loc_arg {
                LocationOrArg::Location(loc) => match self.body.stmt_at(*loc) {
                    Either::Left(Statement { kind, .. }) => match kind {
                        StatementKind::Assign(assign) => Some(assign.0.local),
                        _ => None,
                    },
                    Either::Right(Terminator { kind, .. }) => match kind {
                        TerminatorKind::Call { destination, .. } => Some(destination.local),
                        _ => None,
                    },
                },
                LocationOrArg::Arg(local) => Some(*local),
            })
            .collect()
    }
}

impl<'tcx, 's> Deref for FlowSet<'tcx, 's> {
    type Target = HashSet<LocationOrArg>;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

pub trait BodyFlow<'tcx> {
    fn compute(
        &mut self,
        locals: impl IntoIterator<Item = Local>,
    ) -> BTreeMap<Local, FlowSet<'tcx, '_>>;

    fn flows(&self) -> BTreeMap<Local, FlowSet<'tcx, '_>>;
}

pub struct DirectBodyFlow<'a, 'tcx> {
    full: FullBodyFlow<'a, 'tcx>,
    direct: BTreeMap<Local, HashSet<LocationOrArg>>,
}

impl<'a, 'tcx> BodyFlow<'tcx> for DirectBodyFlow<'a, 'tcx> {
    fn compute(
        &mut self,
        locals: impl IntoIterator<Item = Local>,
    ) -> BTreeMap<Local, FlowSet<'tcx, '_>> {
        let locals: BTreeSet<_> = locals.into_iter().collect();
        self.populate(locals.clone());

        locals
            .into_iter()
            .map(|local| {
                let set = self.direct.get(&local).unwrap();
                (local, FlowSet::new(self.full.body(), set))
            })
            .collect()
    }

    fn flows(&self) -> BTreeMap<Local, FlowSet<'tcx, '_>> {
        self.direct
            .iter()
            .map(|(local, set)| (*local, FlowSet::new(self.full.body(), set)))
            .collect()
    }
}

impl<'a, 'tcx> DirectBodyFlow<'a, 'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, body_id: BodyId) -> Self {
        let full = FullBodyFlow::new(tcx, body_id);
        Self {
            full,
            direct: BTreeMap::new(),
        }
    }

    /// Lazily compute the full flow of `local` (if not already), and prune indirect dependencies.
    fn populate(&mut self, locals: impl IntoIterator<Item = Local>) {
        // filter into locals missing from `direct`.
        let locals: BTreeSet<_> = locals
            .into_iter()
            .filter(|local| !self.direct.contains_key(&local))
            .collect();

        // compute full dependencies.
        let fulls = self.full.compute(locals);

        // compute full dependencies for locals' dependencies.
        let (fulls, all_deps): (BTreeMap<_, _>, Vec<_>) = fulls
            .into_iter()
            .map(|(local, deps)| ((local, deps.set.clone()), deps.assigned_locals()))
            .unzip();
        let all_deps: BTreeSet<_> = all_deps.into_iter().flatten().collect();
        let dep_fulls = self.full.compute(all_deps);

        // populate `direct` map for missing locals.
        for (local, mut direct) in fulls {
            // remove indirect dependencies from `full`.
            for (_, dep_full) in dep_fulls.iter().filter(|(&dep, _)| local != dep) {
                direct = direct.sub(dep_full.set);
            }

            self.direct.insert(local, direct);
        }
    }
}

struct FullBodyFlow<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    body_id: BodyId,
    body_with_facts: &'tcx BodyWithBorrowckFacts<'tcx>,
    results: FlowResults<'a, 'tcx>,

    map: BTreeMap<Local, HashSet<LocationOrArg>>,
}

impl<'a, 'tcx> BodyFlow<'tcx> for FullBodyFlow<'a, 'tcx> {
    fn compute(
        &mut self,
        locals: impl IntoIterator<Item = Local>,
    ) -> BTreeMap<Local, FlowSet<'tcx, '_>> {
        let locals: BTreeSet<_> = locals.into_iter().collect();
        self.populate(locals.clone());

        locals
            .into_iter()
            .map(|local| {
                let set = FlowSet::new(self.body(), self.map.get(&local).unwrap());
                (local, set)
            })
            .collect()
    }

    fn flows(&self) -> BTreeMap<Local, FlowSet<'tcx, '_>> {
        self.map
            .iter()
            .map(|(local, set)| (*local, FlowSet::new(self.body(), set)))
            .collect()
    }
}

impl<'a, 'tcx> FullBodyFlow<'a, 'tcx> {
    fn new(tcx: TyCtxt<'tcx>, body_id: BodyId) -> Self {
        let body_with_facts =
            borrowck_facts::get_body_with_borrowck_facts(tcx, tcx.hir().body_owner_def_id(body_id));
        let results = flowistry::infoflow::compute_flow(tcx, body_id, body_with_facts);
        Self {
            tcx,
            body_id,
            body_with_facts,
            results,
            map: BTreeMap::new(),
        }
    }

    fn populate(&mut self, locals: impl IntoIterator<Item = Local>) {
        // filter into missing locals.
        let locals = locals
            .into_iter()
            .filter(|local| !self.map.contains_key(local));

        // create list of locals and targets.
        let (locals, targets): (Vec<_>, Vec<_>) = locals
            .into_iter()
            .map(|local| {
                let target = self.find_target_of_local(local);
                (local, vec![target])
            })
            .unzip();

        // compute dependencies for targets.
        let deps =
            flowistry::infoflow::compute_dependencies(&self.results, targets, Direction::Forward);

        // filter out location of first assignment.
        let deps: BTreeMap<_, _> = deps
            .into_iter()
            .enumerate()
            .map(|(i, set)| (*locals.get(i).unwrap(), set))
            .map(|(local, set)| {
                let loc = self.find_first_assignment_of_local(local);
                let set: HashSet<_> = set
                    .iter()
                    .filter(|loc_or_arg| match (loc_or_arg, loc) {
                        (LocationOrArg::Location(loc), Some(first)) => *loc != first,
                        (LocationOrArg::Location(_), None) | (LocationOrArg::Arg(_), _) => true,
                    })
                    .cloned()
                    .collect();
                (local, set)
            })
            .collect();

        // populate `full` map.
        self.map.extend(deps);
    }

    fn find_target_of_local(&self, local: Local) -> (Place<'tcx>, LocationOrArg) {
        let place = Place::from_local(local, self.tcx);
        let loc_or_arg = match LocationOrArg::from_place(place, self.body()) {
            Some(loc_or_arg) => loc_or_arg,
            None => {
                let location = self.find_first_assignment_of_local(local).unwrap();
                LocationOrArg::Location(location)
            }
        };
        (place, loc_or_arg)
    }

    fn find_first_assignment_of_local(&self, local: Local) -> Option<Location> {
        fn h<'tcx>(body: &Body<'tcx>, local: Local, location: Location) -> Option<Location> {
            // ensure that location is valid.
            if location.block.as_usize() >= body.basic_blocks.len() {
                return None;
            }

            // iterate through body until location where local is first assigned.
            let location = match body.stmt_at(location) {
                Either::Left(Statement { kind, .. }) => match kind {
                    // if same local, found!
                    StatementKind::Assign(assign) if assign.0.local == local => {
                        return Some(location);
                    }
                    _ => location.successor_within_block(),
                },
                Either::Right(Terminator { kind, .. }) => match kind {
                    TerminatorKind::Call { destination, .. } if destination.local == local => {
                        return Some(location)
                    }
                    _ => Location {
                        block: location.block + 1,
                        statement_index: 0,
                    },
                },
            };

            h(body, local, location)
        }

        h(self.body(), local, Location::START)
    }

    fn body(&self) -> &'tcx Body<'tcx> {
        &self.body_with_facts.body
    }
}

fn collect_set(set: LocationOrArgSet) -> HashSet<LocationOrArg> {
    set.iter().cloned().collect()
}
