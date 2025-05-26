use deno_core::anyhow::Error;
use std::rc::Rc;
use crate::Args;

pub async fn run_js(args: &Args) -> Result<(), Error> {
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    // Load our runtime.js
    let mod_runtime = js_runtime
      .load_side_es_module_from_code(
        &deno_core::ModuleSpecifier::parse("runjs:runtime.js")?,
      include_str!("../scripts/runtime.js"),
      )
      .await?;
    let mod_runtime_result = js_runtime.mod_evaluate(mod_runtime);

    // Load our main module
    let abs_vtt = std::path::absolute(std::path::Path::new(&args.vtt))?;
    let abs_vtt_path = abs_vtt.as_path();
    let foundry = deno_core::resolve_path("./main.js", abs_vtt_path)?;
    let mod_foundry = js_runtime.load_main_es_module(&foundry).await?;
    let mod_foundry_result = js_runtime.mod_evaluate(mod_foundry);

    // Await all runtimes
    js_runtime.run_event_loop(Default::default()).await?;
    mod_runtime_result.await?;
    mod_foundry_result.await.expect("Core error");

    // We're done here
    Ok(())
}