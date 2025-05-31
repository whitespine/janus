use std::fs::read_to_string;
use crate::error::ScriptError;
use crate::Args;
use rustyscript::{json_args, Error, Module, ModuleHandle, Runtime, RuntimeOptions};
use std::time::Duration;
use clap::Arg;

fn vtt_path(args: &Args, subpath: &str) -> std::path::PathBuf {
    let vtt = std::path::Path::new(&args.vtt).join(subpath);
    std::path::absolute(vtt).expect("Bad Path") // We don't do this dynamically
}

fn vtt_module(args: &Args, name: &str, subpath: &str) -> Module {
    let vtt_path = format!("{}/public/scripts/{}", args.vtt, subpath);
    Module::new(
        name,
        read_to_string(vtt_path).expect(&format!("Failed to read path {}", subpath)),
    )
}

pub struct FoundryRuntime {
    runtime: Runtime,
    core: ModuleHandle
}

impl FoundryRuntime {
    pub fn new(args: &Args, tokio_runtime: std::rc::Rc<tokio::runtime::Runtime>) -> FoundryRuntime {
        let mut runtime = Runtime::with_tokio_runtime(RuntimeOptions {
            timeout: Duration::from_millis(500), // Stop execution by force after 50ms
            ..Default::default()
        }, tokio_runtime).expect("Failed to create runtime");

        // Load our core extension and foundry capabilities
        let core = Module::new( "main.js", include_str!("../scripts/main.js"));
        let shim = Module::new("shim.js", include_str!("../scripts/shim.js"));
        let foundry = vtt_module(args, "foundry.mjs", "foundry.mjs");
        let vendor = vtt_module(args, "./vendor.mjs", "vendor.mjs");
        let core_handle = runtime.load_modules(&core, vec![&shim, &vendor, &foundry]).expect("Failed to load core");

        // Load our foundry extension
        FoundryRuntime {
            core: core_handle,
            runtime
        }
    }

    pub async fn run_in_foundry(&mut self, expression: &str) -> Result<String, ScriptError> {
        match self.runtime.eval_async::<String>(expression).await {
            Ok(v) => Ok(v),
            Err(rse) => Err(ScriptError::JsError(rse))
        }
    }
}