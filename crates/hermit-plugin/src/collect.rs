use rustc_ast::{
    token::{Lit, LitKind, Token, TokenKind},
    tokenstream::TokenTree,
    AttrKind, Attribute,
};
use rustc_borrowck::consumers::BodyWithBorrowckFacts;
use rustc_hir::{BodyId, ItemKind};
use rustc_lexer::unescape;
use rustc_middle::{
    hir::map::Map as HirMap,
    mir::{Local, Place},
    ty::TyCtxt,
};
use rustc_span::{def_id::LocalDefId, Span};
use rustc_utils::{
    mir::location_or_arg::LocationOrArg,
    source_map::spanner::{EnclosingHirSpans, Spanner},
    BodyExt, PlaceExt, SpanExt,
};

use flowistry::infoflow::Direction;
use hermit_syntax::{
    attribute::{AgentMeta, Decode, EnsureMeta, ForgetMeta, HaveMeta},
    TOOL,
};

use crate::meta;

pub struct Collector<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> Collector<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self { tcx }
    }

    pub fn collect(&self) -> meta::Info {
        let hir = self.tcx.hir();
        let fns = hir
            .items()
            .filter_map(|id| match hir.item(id).kind {
                ItemKind::Fn(_, _, body_id) => Some(body_id),
                _ => None,
            })
            .map(|body_id| FnCollector::new(self.tcx, body_id))
            .map(|fnc| fnc.collect())
            .collect();

        meta::Info { fns }
    }
}

struct FnCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    body_id: BodyId,
}

enum AttrInfo {
    Agent(meta::Agent),
    Have(meta::Have),
    Ensure(meta::Ensure),
    Forget(meta::Forget),
}

impl<'tcx> FnCollector<'tcx> {
    fn new(tcx: TyCtxt<'tcx>, body_id: BodyId) -> Self {
        Self { tcx, body_id }
    }

    fn hir(&self) -> HirMap<'tcx> {
        self.tcx.hir()
    }

    fn def_id(&self) -> LocalDefId {
        self.hir().body_owner_def_id(self.body_id)
    }

    fn attrs(&self) -> &'tcx [Attribute] {
        self.tcx.get_attrs_unchecked(self.def_id().into())
    }

    fn collect(&self) -> meta::Function {
        // collect the attributes.
        let mut agents = Vec::new();
        let mut haves = Vec::new();
        let mut ensures = Vec::new();
        let mut forgets = Vec::new();
        let attrs = self
            .attrs()
            .iter()
            .filter_map(|attr| self.collect_attr(attr));
        attrs.for_each(|attr| match attr {
            AttrInfo::Agent(attr) => agents.push(attr),
            AttrInfo::Have(attr) => haves.push(attr),
            AttrInfo::Ensure(attr) => ensures.push(attr),
            AttrInfo::Forget(attr) => forgets.push(attr),
        });

        // let body_with_facts = borrowck_facts::get_body_with_borrowck_facts(self.tcx, def_id);
        meta::Function {
            agents,
            haves,
            ensures,
            forgets,
            calls: vec![],
        }
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

        if kind == AgentMeta::KIND {
            Some(self.collect_agent_attr(&arg))
        } else if kind == HaveMeta::KIND {
            Some(self.collect_have_attr(&arg))
        } else if kind == EnsureMeta::KIND {
            Some(self.collect_ensure_attr(&arg))
        } else if kind == ForgetMeta::KIND {
            Some(self.collect_forget_attr(&arg))
        } else {
            panic!()
        }
    }

    fn collect_agent_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Agent(AgentMeta::decode(arg))
    }

    fn collect_have_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Have(HaveMeta::decode(arg))
    }

    fn collect_ensure_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Ensure(EnsureMeta::decode(arg))
    }

    fn collect_forget_attr(&self, arg: &str) -> AttrInfo {
        AttrInfo::Forget(ForgetMeta::decode(arg))
    }
}

fn compute_dependencies<'tcx>(
    tcx: TyCtxt<'tcx>,
    body_id: BodyId,
    body_with_facts: &BodyWithBorrowckFacts<'tcx>,
) {
    println!("Body:\n{}", body_with_facts.body.to_string(tcx).unwrap());

    // This computes the core information flow data structure. But it's not very
    // visualizable, so we need to post-process it with a specific query.
    let results = flowistry::infoflow::compute_flow(tcx, body_id, body_with_facts);

    // We construct a target of the first argument at the start of the function.
    let arg_local = Local::from_usize(1);
    let arg_place = Place::make(arg_local, &[], tcx);
    let targets = vec![vec![(arg_place, LocationOrArg::Arg(arg_local))]];

    // Then use Flowistry to compute the locations and places influenced by the target.
    let location_deps =
        flowistry::infoflow::compute_dependencies(&results, targets.clone(), Direction::Forward)
            .remove(0);

    // And print out those forward dependencies. Note that while each location has an
    // associated span in the body, i.e. via `body.source_info(location).span`,
    // these spans are pretty limited so we have our own infrastructure for mapping MIR
    // back to source. That's the Spanner class and the location_to_span method.
    println!("The forward dependencies of targets {targets:?} are:");
    let body = &body_with_facts.body;
    let spanner = Spanner::new(tcx, body_id, body);
    let source_map = tcx.sess.source_map();
    for location in location_deps.iter() {
        let spans = Span::merge_overlaps(spanner.location_to_spans(
            *location,
            body,
            EnclosingHirSpans::OuterOnly,
        ));
        println!("Location {location:?}:");
        for span in spans {
            println!("{}", source_map.span_to_snippet(span).unwrap(),);
        }
    }
}
