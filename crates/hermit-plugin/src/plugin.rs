use std::borrow::Cow;
use std::env;
use std::process::Command;

use clap::Parser;
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, Utf8Path};
use rustc_utils::mir::borrowck_facts;
use serde::{Deserialize, Serialize};

use crate::collect::Collector;

pub struct HermitPlugin;

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct HermitPluginArgs {
    #[clap(last = true)]
    cargo_args: Vec<String>,
}

#[derive(Debug)]
pub struct HermitPluginConfig {}

impl From<HermitPluginArgs> for HermitPluginConfig {
    fn from(value: HermitPluginArgs) -> Self {
        let HermitPluginArgs { cargo_args: _ } = value;
        Self {}
    }
}

impl RustcPlugin for HermitPlugin {
    type Args = HermitPluginArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "hermit-driver".into()
    }

    fn args(&self, _target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args> {
        let args = HermitPluginArgs::parse_from(env::args().skip(1));
        let filter = CrateFilter::AllCrates;
        RustcPluginArgs { args, filter }
    }

    fn modify_cargo(&self, cargo: &mut Command, args: &Self::Args) {
        cargo.args(&args.cargo_args);
    }

    fn run(
        self,
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        let mut callbacks = HermitCallbacks {
            config: plugin_args.into(),
        };
        let compiler = rustc_driver::RunCompiler::new(&compiler_args, &mut callbacks);
        compiler.run()
    }
}

#[derive(Debug)]
struct HermitCallbacks {
    config: HermitPluginConfig,
}

impl rustc_driver::Callbacks for HermitCallbacks {
    fn config(&mut self, config: &mut rustc_interface::Config) {
        // You MUST configure rustc to ensure `get_body_with_borrowck_facts` will work.
        borrowck_facts::enable_mir_simplification();
        config.override_queries = Some(borrowck_facts::override_queries);
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        queries.global_ctxt().unwrap().enter(|tcx| {
            let coll = Collector::new(tcx);
            let info = coll.collect();
        });

        rustc_driver::Compilation::Stop
    }
}
