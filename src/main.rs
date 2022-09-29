extern crate zenith;
extern crate zenith_derive;

use glob::glob;
use std::{collections::HashMap, path::Path};
use wasmer::{Instance, Module, Store, TypedFunction};
use wasmer_wasi::WasiState;
use zenith::prelude::*;

type ModuleCache = HashMap<String, Module>;

const WASM_PATH: &str = "D:\\Jesse\\repo\\@zenith\\zenith-wasm\\apps\\wasm";

// TODO: Write variant that uses Wasmer Memory{} instead of memory defined in the module
fn call_app_raw(
    mut store: Store,
    modules: &ModuleCache,
    app_name: &str,
    data: &[u8],
) -> anyhow::Result<Store> {
    let module = modules.get(app_name);

    match module {
        Some(app) => {
            let wasi_env = WasiState::new(app_name).finalize(&mut store)?;
            let imports = wasi_env.import_object_for_all_wasi_versions(&mut store, &app)?;

            // Create module instance
            let instance = Instance::new(&mut store, &app, &imports)?;
            let memory = instance.exports.get_memory("memory")?;
            wasi_env.data_mut(&mut store).set_memory(memory.clone());

            // Retrieve WASM memory utility functions
            let get_input_max_size: TypedFunction<(), i32> = instance
                .exports
                .get_typed_function(&mut store, "get_input_buffer_size")?;
            let write_input: TypedFunction<(i32, i32), i32> = instance
                .exports
                .get_typed_function(&mut store, "write_input")?;
            let read_output: TypedFunction<i32, i32> = instance
                .exports
                .get_typed_function(&mut store, "read_output")?;
            let run: TypedFunction<(i32, i32), i32> = instance
                .exports
                .get_typed_function(&mut store, "__zen_run")?;

            // Ensure data fits in module input buffer
            let input_size = data.len() as i32;
            let input_offset = 0;
            let input_max_size = get_input_max_size.call(&mut store)?;
            if input_size > input_max_size {
                println!("Input exceeds module's input buffer");
                panic!();
            };

            // Write input to module's input buffer
            for (i, byte) in data.iter().enumerate() {
                let val = *byte;
                let _ = write_input.call(&mut store, i as i32, val as i32)?;
            }
            let bytes_written = run.call(&mut store, input_offset, input_size)?;
            println!("Module wrote {:?} bytes to output", bytes_written);

            // Copy WASM output buffer
            let mut output = vec![0 as u8; bytes_written as usize];
            for i in 0..bytes_written {
                output[i as usize] = read_output.call(&mut store, i as i32)? as u8;
            }

            let output_str = std::str::from_utf8(output.as_slice())?;
            println!("Output: {}", output_str);

            Ok(store)
        }
        None => {
            println!("Cannot find module");
            panic!();
        }
    }
}

fn main() {
    let data = Input {
        name: "Jesse".to_string(),
    };
    let path_pattern = Path::new(WASM_PATH).join("*.wasm");

    let store = Store::default();
    let mut modules = HashMap::<String, Module>::new();
    let entries = glob(path_pattern.to_str().unwrap()).unwrap();

    for entry in entries {
        match entry {
            Ok(path) => {
                let app_name = path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(".wasm", "");

                let module = Module::from_file(&store, path).unwrap();
                println!("  => Found module {}", app_name);

                modules.insert(app_name.to_string(), module);
            }
            Err(_err) => panic!(),
        }
    }

    call_app_raw(
        store,
        &modules,
        "greeter",
        bincode::serialize(&data).unwrap().as_slice(),
    )
    .unwrap();
}
