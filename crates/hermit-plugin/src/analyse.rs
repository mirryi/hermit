use rustc_middle::ty::TyCtxt;

use crate::collect::Collector;

pub fn analyse<'tcx>(tcx: TyCtxt<'tcx>) {
    let coll = Collector::new(tcx);
    let info = coll.collect();

    // println!("{:#?}", info);
}
