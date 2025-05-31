use std::fs;
use url::Url;
use crate::Args;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use deno_core::error::AnyError;
use deno_core::op2;
use deno_core::FsModuleLoader;
use deno_core::ModuleSpecifier;
use deno_fs::RealFs;
use deno_resolver::npm::DenoInNpmPackageChecker;
use deno_resolver::npm::NpmResolver;
use deno_runtime::deno_core::{JsRuntime, ModuleSpecifier};
use deno_runtime::deno_core::error::AnyError;
use deno_runtime::deno_permissions::PermissionsContainer;
use deno_runtime::permissions::RuntimePermissionDescriptorParser;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use deno_runtime::worker::WorkerServiceOptions;

fn vtt_path(args: &Args, subpath: &str) -> std::path::PathBuf {
    let vtt = std::path::Path::new(&args.vtt).join(subpath);
    std::path::absolute(vtt).expect("Bad Path") // We don't do this dynamically
}

async fn load_script(runtime: &mut JsRuntime, specifier: &str, script: String) -> Result<(), Error> {
    let shim = runtime
        .load_side_es_module_from_code(
            &deno_core::ModuleSpecifier::parse(specifier)?,
            script
        )
        .await?;
    runtime.mod_evaluate(shim).await?;
    Ok(())
}

async fn load_vtt(runtime: &mut JsRuntime, specifier: &str, path: std::path::PathBuf) -> Result<(), Error> {
    let script = fs::read_to_string(path)?;
    load_script(runtime, specifier, script).await
}

#[op2(fast)]
fn op_hello(#[string] text: &str) {
    println!("Hello {} from an op!", text);
}

deno_core::extension!(
  hello_runtime,
  ops = [op_hello],
  esm_entry_point = "ext:hello_runtime/bootstrap.js",
  esm = [dir "examples/extension", "bootstrap.js"]
);

pub async fn run_js(args: &Args) -> Result<(), AnyError> {
    // Load our shim.js
    // load_script(&mut js_runtime, "voyeur:scripts/shim.js", include_str!("../scripts/shim.js").to_string()).await?;

    // Load all other deps from vtt
    /*
    load_vtt(&mut js_runtime,
             &format!("{}/scripts/vendor.mjs", args.host),
             vtt_path(args, "./public/scripts/vendor.mjs")).await?;
    load_vtt(&mut js_runtime,
             &format!("{}/scripts/foundry.mjs", args.host),
             vtt_path(args, "./public/scripts/foundry.mjs")).await?;

     */
    let js_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/extension/main.js");
    let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();
    eprintln!("Running {main_module}...");
    let fs = Arc::new(RealFs);
    let permission_desc_parser = Arc::new(
        RuntimePermissionDescriptorParser::new(sys_traits::impls::RealSys),
    );

    let mut worker = MainWorker::bootstrap_from_options(
        &main_module,
        WorkerServiceOptions::<
            DenoInNpmPackageChecker,
            NpmResolver<sys_traits::impls::RealSys>,
            sys_traits::impls::RealSys,
        > {
            module_loader: Rc::new(FsModuleLoader),
            permissions: PermissionsContainer::allow_all(permission_desc_parser),
            blob_store: Default::default(),
            broadcast_channel: Default::default(),
            feature_checker: Default::default(),
            node_services: Default::default(),
            npm_process_state_provider: Default::default(),
            root_cert_store_provider: Default::default(),
            fetch_dns_resolver: Default::default(),
            shared_array_buffer_store: Default::default(),
            compiled_wasm_module_store: Default::default(),
            v8_code_cache: Default::default(),
            fs,
        },
        WorkerOptions {
            extensions: vec![hello_runtime::init_ops_and_esm()],
            ..Default::default()
        },
    );
    worker.execute_main_module(&main_module).await?;
    worker.run_event_loop(false).await?;

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