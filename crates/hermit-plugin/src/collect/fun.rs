use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use rustc_ast::{
    token::{Lit, LitKind, Token, TokenKind},
    tokenstream::TokenTree,
    AttrKind, Attribute,
};
use rustc_borrowck::consumers::BodyWithBorrowckFacts;
use rustc_hir::BodyId;
use rustc_lexer::unescape;
use rustc_middle::mir::{Local, Location};
use rustc_middle::{
    hir::map::Map as HirMap,
    mir::{
        Body, Place, Statement, StatementKind, Terminator, TerminatorKind, VarDebugInfo,
        VarDebugInfoContents,
    },
    ty::TyCtxt,
};
use rustc_span::def_id::LocalDefId;
use rustc_utils::{
    mir::{
        borrowck_facts,
        location_or_arg::{index::LocationOrArgSet, LocationOrArg},
    },
    PlaceExt,
};

use either::Either;
use flowistry::infoflow::{Direction, FlowResults};
use itertools::Itertools;

use hermit_syntax::{
    attribute::{
        AgentMeta as AgentAttribute, Decode as DecodeAttribute, EnsureMeta as EnsureAttribute,
        ForgetMeta as ForgetAttribute, HaveMeta as HaveAttribute,
    },
    TOOL,
};

use crate::meta;

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
        // collect all location.
        let locs = self
            .body()
            .var_debug_info
            .iter()
            .filter_map(|VarDebugInfo { name, value, .. }| {
                let name = name.as_str();
                match value {
                    VarDebugInfoContents::Place(place) => {
                        Some((name, meta::FunctionLocation(place.local)))
                    }
                    VarDebugInfoContents::Const(_) => None,
                }
            })
            .unique_by(|(name, _)| *name)
            .collect::<BTreeMap<_, _>>();

        println!("{:#?}", self.def_id());
        println!("{:#?}", self.body().var_debug_info);
        println!("{:#?}", locs);

        // for each location, compute the local forward dependencies.
        let flows = self.collect_flows(locs.into_iter().map(|(_, loc)| loc));

        // TODO collect the attributes.
        let agents = Vec::new();
        let haves = Vec::new();
        let ensures = Vec::new();
        let forgets = Vec::new();
        let attrs: Vec<_> = self
            .attrs()
            .iter()
            .filter_map(|attr| self.collect_attr(attr))
            .collect();

        // for attr in attrs {
        // match attr {
        // AttrInfo::Agent(ann) => agents.push(ann),
        // AttrInfo::Have(ann) => haves.push(ann),
        // AttrInfo::Ensure(ann) => ensures.push(ann),
        // AttrInfo::Forget(ann) => forgets.push(ann),
        // }
        // }

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

    /// Collect the flows of `places`.
    fn collect_flows(
        &self,
        locs: impl IntoIterator<Item = meta::FunctionLocation>,
    ) -> BTreeMap<meta::FunctionLocation, Vec<meta::Call>> {
        let flow_results =
            flowistry::infoflow::compute_flow(self.tcx, self.body_id, self.body_with_facts);

        locs.into_iter()
            .map(|loc| (loc, self.collect_flow(loc, &flow_results)))
            .collect()
    }

    /// Collect the forward flow of `local`.
    fn collect_flow(
        &self,
        loc: meta::FunctionLocation,
        flow_results: &FlowResults<'_, 'tcx>,
    ) -> Vec<meta::Call> {
        let flow = compute_flow(
            self.tcx,
            self.body(),
            flow_results,
            vec![loc.0],
            Direction::Forward,
        )
        .remove(0);

        // collect the tainted locals in this body.
        let tainted: BTreeSet<_> = flow
            .iter()
            .filter_map(|loc_arg| match loc_arg {
                LocationOrArg::Location(loc) => match self.body().stmt_at(*loc) {
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
            .collect();

        // collect the dependent calls to other functions.
        let calls = flow
            .iter()
            .filter_map(|loc_arg| match loc_arg {
                LocationOrArg::Arg(_) => None,
                LocationOrArg::Location(loc) => match self.body().stmt_at(*loc) {
                    Either::Left(_) => None,
                    Either::Right(Terminator { kind, .. }) => match kind {
                        TerminatorKind::Call { func, args, .. } => {
                            // handle only const function definitions
                            let (id, _) = func.const_fn_def()?;

                            // for each argument in the taint set, map to a call dependency.
                            let calls = args
                                .into_iter()
                                .enumerate()
                                .filter_map(|(i, arg)| arg.place().map(|place| (i, place.local)))
                                .filter(|(_, local)| tainted.contains(local))
                                .map(move |(i, _)| meta::Call {
                                    fun: meta::FunctionId(id),
                                    idx: i,
                                });
                            Some(calls)
                        }
                        _ => None,
                    },
                },
            })
            .flatten()
            .collect();

        calls
    }

    fn collect_attr(&self, attr: &Attribute) -> Option<AttrInfo> {
        let normal = match &attr.kind {
            AttrKind::Normal(normal) => normal,
            AttrKind::DocComment(_, _) => return None,
        };

        let segments = &normal.item.path.segments;
        let tool = segments.first()?.ident.as_str();
        let kind = segments.get(1)?.ident.as_str();
        let none = segments.get(2);
        if !(tool == TOOL.name() && none.is_none()) {
            return None;
        }

        // extract the string argument.
        let mut args = normal.item.args.inner_tokens().into_trees();
        let arg = match args.next_ref().unwrap() {
            TokenTree::Token(
                Token {
                    kind:
                        TokenKind::Literal(Lit {
                            kind: LitKind::Str,
                            symbol,
                            suffix: _,
                        }),
                    span: _,
                },
                _,
            ) => {
                let mut buf = String::new();
                unescape::unescape_literal(symbol.as_str(), unescape::Mode::Str, &mut |_, c| {
                    buf.push(c.unwrap())
                });
                buf
            }
            _ => panic!(),
        };

        if kind == AgentAttribute::KIND {
            Some(self.collect_agent_attr(&arg))
        } else if kind == HaveAttribute::KIND {
            Some(self.collect_have_attr(&arg))
        } else if kind == EnsureAttribute::KIND {
            Some(self.collect_ensure_attr(&arg))
        } else if kind == ForgetAttribute::KIND {
            Some(self.collect_forget_attr(&arg))
        } else {
            panic!()
        }
    }

    fn collect_agent_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Agent(AgentAttribute::decode(arg))
    }

    fn collect_have_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Have(HaveAttribute::decode(arg))
    }

    fn collect_ensure_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Ensure(EnsureAttribute::decode(arg))
    }

    fn collect_forget_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Forget(ForgetAttribute::decode(arg))
    }
}

enum AttrInfo {
    Agent(AgentAttribute),
    Have(HaveAttribute),
    Ensure(EnsureAttribute),
    Forget(ForgetAttribute),
}

impl AttrInfo {
    fn variables(&self) -> impl Iterator<Item = &hermit_syntax::lang::Ident> {
        let iter: Box<dyn Iterator<Item = _>> = match self {
            AttrInfo::Agent(_) => Box::new(iter::empty()),
            AttrInfo::Have(HaveAttribute { form }) => Box::new(form.0.vocab()),
            AttrInfo::Ensure(EnsureAttribute { form }) => Box::new(form.0.vocab()),
            AttrInfo::Forget(ForgetAttribute {
                subject,
                dependencies,
            }) => Box::new(iter::once(subject).chain(dependencies).map(|var| &var.0)),
        };
        iter
    }
}

/// Compute the directional dependencies of the `places`.
fn compute_flow<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    flow_results: &FlowResults<'_, 'tcx>,
    locals: impl IntoIterator<Item = Local>,
    direction: Direction,
) -> Vec<LocationOrArgSet> {
    let targets = locals
        .into_iter()
        .map(|local| {
            vec![(
                Place::from_local(local, tcx),
                location_or_arg_of_local(tcx, body, local),
            )]
        })
        .collect::<Vec<_>>();
    flowistry::infoflow::compute_dependencies(&flow_results, targets, direction)
}

/// Create a flow target from `local`.
fn location_or_arg_of_local<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    local: Local,
) -> LocationOrArg {
    let place = Place::from_local(local, tcx);
    match LocationOrArg::from_place(place, body) {
        Some(loc_or_arg) => loc_or_arg,
        None => {
            let location = location_of_local(body, local).unwrap();
            LocationOrArg::Location(location)
        }
    }
}

/// Find the location where `local` is first assigned.
fn location_of_local<'tcx>(body: &Body<'tcx>, local: Local) -> Option<Location> {
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

    h(body, local, Location::START)
}
