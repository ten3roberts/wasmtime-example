use anyhow::Context;
use std::error::Error;
use wasmtime::{component::Component, *};

static GUEST_BYTES: &[u8] = include_bytes!("../bin/guest.wasm");

struct Host;

fn main() -> Result<(), Box<dyn Error>> {
    use tracing_subscriber::{prelude::*, registry, EnvFilter};
    use tracing_tree::HierarchicalLayer;

    registry()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_indent_lines(true)
                .with_span_retrace(true),
        )
        .init();

    let mut config = Config::default();
    config
        .wasm_component_model(true)
        .wasm_backtrace(true)
        .wasm_backtrace_details(WasmBacktraceDetails::Enable);

    let engine = Engine::new(&config)?;

    tracing::info!("create store");
    let mut store = Store::new(&engine, Host);

    tracing::info!("create component");
    let component = Component::new(&engine, GUEST_BYTES)?;

    // Create a linker that will be used to resolve the component's imports, if any.
    let mut linker = component::Linker::new(&engine);

    tracing::info!("Defining imports");

    let mut root = linker.root();

    root.func_wrap("print", |_: StoreContextMut<'_, Host>, msg: (String,)| {
        tracing::info!(target: "guest", "{msg:?}");
        Ok(())
    })
    .unwrap();

    root.func_wrap("get-value", |_, (key,): (u32,)| {
        Ok((((key as u64) * (key as u64), (key as f32).sqrt()),))
    })
    .unwrap();

    // Create an instance of the component using the linker.
    // Main::add_to_linker(&mut linker, |v: &mut Host| v)?;
    // let (bindings, _) = Main::instantiate(&mut store, &component, &linker)?;
    let instance = linker.instantiate(&mut store, &component)?;

    tracing::info!("Finished instantiating component");

    let func_run;

    let func_get_name;

    {
        let mut exports = instance.exports(&mut store);
        let mut interface = exports.root();
        func_run = interface.typed_func::<(Vec<String>,), (Result<i32, String>,)>("run")?;
        func_get_name = interface.typed_func::<(), (String,)>("get-name")?;
    }

    tracing::info!("Calling run");

    tracing::info_span!("run").in_scope(|| {
        let (result,) = func_run
            .call(&mut store, (vec!["guest".into(), "Hello".into()],))
            .context("Failed to call `run`")?;

        func_run.post_return(&mut store)?;

        tracing::info!(?result, "result");

        assert_eq!(result, Ok(42));

        anyhow::Ok(())
    })?;

    tracing::info_span!("get-name").in_scope(|| {
        let (result,) = func_get_name
            .call(&mut store, ())
            .context("Failed to call `get-name`")?;
        tracing::info!("received name: {result:?}");

        func_get_name.post_return(&mut store)?;

        assert_eq!(result, "guest-module");

        anyhow::Ok(())
    })?;

    Ok(())
}
