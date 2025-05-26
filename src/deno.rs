use std::fs;
use deno_core::anyhow::Error;
use std::rc::Rc;
use url::Url;
use crate::Args;

fn vtt_path(args: &Args, subpath: &str) -> Url {
    let vtt = std::path::Path::new(&args.vtt);
    let abs_vtt = std::path::absolute(vtt).expect("Bad Path"); // We don't do this dynamically
    deno_core::resolve_path(subpath, abs_vtt.as_path()).expect("Bad Url")
}

async fn load_script(runtime: &mut deno_core::JsRuntime, specifier: &str, script: String) -> Result<(), Error> {
    let shim = runtime
        .load_side_es_module_from_code(
            &deno_core::ModuleSpecifier::parse(specifier)?,
            script
        )
        .await?;
    runtime.mod_evaluate(shim).await?;
    Ok(())
}

async fn load_vtt(runtime: &mut deno_core::JsRuntime, path: &str) -> Result<(), Error> {
    // load_script(runtime, )
    todo!()
}

pub async fn run_js(args: &Args) -> Result<(), Error> {
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    // Load our shim.js
    load_script(&mut js_runtime, "voyeur:scripts/shim.js", include_str!("../scripts/shim.js").to_string()).await?;

    // Load all other deps from vtt

    // Load our main module
    let mod_foundry = js_runtime.load_main_es_module_from_code(
        &deno_core::ModuleSpecifier::parse("voyeur:scripts/main.js")?,
        include_str!("../scripts/main.js").to_string()
    ).await?;
    js_runtime.mod_evaluate(mod_foundry).await?;

    // Await all runtimes
    js_runtime.run_event_loop(Default::default()).await?;

    // We're done here
    Ok(())
}