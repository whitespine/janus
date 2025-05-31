use crate::error::ScriptError;
use crate::Args;
use rustyscript::{json_args, Error, Module, ModuleHandle, Runtime, RuntimeOptions};
use std::time::Duration;

fn vtt_path(args: &Args, subpath: &str) -> std::path::PathBuf {
    let vtt = std::path::Path::new(&args.vtt).join(subpath);
    std::path::absolute(vtt).expect("Bad Path") // We don't do this dynamically
}

pub struct FoundryRuntime {
    runtime: Runtime,
    core: ModuleHandle
}

impl FoundryRuntime {
    pub fn new(_args: &Args, tokio_runtime: std::rc::Rc<tokio::runtime::Runtime>) -> FoundryRuntime {
        let mut runtime = Runtime::with_tokio_runtime(RuntimeOptions {
            timeout: Duration::from_millis(500), // Stop execution by force after 50ms
            ..Default::default()
        }, tokio_runtime).expect("Failed to create runtime");
        let core = Module::new(
            "main.js", include_str!("../scripts/main.js")
        );
        let core_handle = runtime.load_module(&core).expect("Failed to load core");
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