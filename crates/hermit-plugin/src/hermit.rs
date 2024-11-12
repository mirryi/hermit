extern crate rustc_middle;

use flowistry::infoflow::FlowResults;
use hermit_core::Flow as HermitFlowAnalysis;
use rustc_middle::mir::Location;

pub struct FlowAnalysis<'a, 'tcx> {
    results: FlowResults<'a, 'tcx>,
}

impl<'a, 'tcx> HermitFlowAnalysis for FlowAnalysis<'a, 'tcx> {
    type Location = Location;

    fn flow(&self, name: &Self::Location) -> Vec<Self::Location> {
        todo!()
    }
}
