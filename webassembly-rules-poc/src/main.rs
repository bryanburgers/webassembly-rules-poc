use clap::Parser;
use std::{borrow::Cow, path::PathBuf};

/// The struct that represents command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The validator in WebAssembly format
    #[arg(short, long, value_name = "FILE")]
    webassembly: PathBuf,

    /// The path to the JSON data
    #[arg(short, long, value_name = "FILE")]
    data: PathBuf,

    /// The path to the JSON previous data
    ///
    /// If this is not supplied, null will be provided as the previous data.
    #[arg(short, long, value_name = "FILE")]
    previous_data: Option<PathBuf>,

    /// Turn debugging information on
    ///
    /// Use once to get any wasm calls to the `diagnostic` host call. Use twice to output detailed
    /// information about all host calls.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

/// Quick little helper that helps with logging host calls.
macro_rules! log_call {
    ($context:expr, $($rest:tt)*) => {
        if $context.verbose >= 2 {
            println!("\x1b[3;90m{}\x1b[0m", format_args!($($rest)*));
        }
    };
}

fn main() {
    // Parse the command line arguments
    let args = Args::parse();
    // Build up a context based on the arguments
    let context = Context::from_args(&args);

    // Base wasmtime code we need for a new engine.
    let config = wasmtime::Config::new();
    let engine = match wasmtime::Engine::new(&config) {
        Ok(engine) => engine,
        Err(err) => {
            eprintln!("Failed to create engine: {err}");
            std::process::exit(1);
        }
    };

    // Parse the module from the passed in webassembly.
    let module = match wasmtime::Module::from_file(&engine, &args.webassembly) {
        Ok(module) => module,
        Err(err) => {
            eprintln!(
                "Failed to create module from {}: {err}",
                args.webassembly.to_string_lossy(),
            );
            std::process::exit(2);
        }
    };

    // Instantiate a new instance.
    let mut store = wasmtime::Store::new(&engine, context);
    let linker = create_linker(&engine, &module);
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(instance) => instance,
        Err(err) => {
            eprintln!("Failed to instantiate module: {err}");
            std::process::exit(3);
        }
    };

    // Find the validate function in the module.
    let function = match instance.get_typed_func::<(), ()>(&mut store, "validate") {
        Ok(function) => function,
        Err(err) => {
            eprintln!("Failed to get `validate` function from WebAssembly module: {err}");
            std::process::exit(4);
        }
    };

    // And call the validate function!
    match function.call(&mut store, ()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Execution failed: {err}");
            std::process::exit(5);
        }
    }

    // That's it. We're done.
    let context = store.into_data();
    log_call!(context, "Validation program finished");
}

struct Context {
    /// Stringified version of the JSON for the data
    pub data: String,
    /// Stringified version of the JSON for the previous data
    pub previous_data: String,
    /// Which verbosity level we're at
    pub verbose: u8,
}

impl Context {
    pub fn from_args(args: &Args) -> Self {
        let data_contents = match std::fs::read(&args.data) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!(
                    "Failed to read JSON from '{}': {err}",
                    args.data.to_string_lossy()
                );
                std::process::exit(1);
            }
        };
        let data: serde_json::Value = match serde_json::from_slice(&data_contents) {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Contents of '{}' was not JSON: {err}",
                    args.data.to_string_lossy()
                );
                std::process::exit(1);
            }
        };

        let previous_data = if let Some(previous_data) = &args.previous_data {
            let previous_data_contents = match std::fs::read(previous_data) {
                Ok(contents) => contents,
                Err(err) => {
                    eprintln!(
                        "Failed to read JSON from '{}': {err}",
                        previous_data.to_string_lossy()
                    );
                    std::process::exit(1);
                }
            };
            let previous_data: serde_json::Value =
                match serde_json::from_slice(&previous_data_contents) {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!(
                            "Contents of '{}' was not JSON: {err}",
                            previous_data.to_string_lossy()
                        );
                        std::process::exit(1);
                    }
                };
            previous_data
        } else {
            serde_json::Value::Null
        };

        Self {
            data: serde_json::to_string(&data).unwrap(),
            previous_data: serde_json::to_string(&previous_data).unwrap(),
            verbose: args.verbose,
        }
    }
}

/// Define all of the host functions that the module can call
fn create_linker(
    engine: &wasmtime::Engine,
    module: &wasmtime::Module,
) -> wasmtime::Linker<Context> {
    let mut linker = wasmtime::Linker::new(engine);

    // reso.data – fill the provided buffer with UTF-8-encoded JSON data. If there is more data
    // than the module has room for, do nothing and just return the size of the JSON data.
    linker
        .func_wrap(
            "reso",
            "data",
            |mut caller: wasmtime::Caller<'_, Context>,
             len: i32,
             ptr: i32|
             -> wasmtime::Result<i32> {
                let Some(memory) = caller
                    .get_export("memory")
                    .and_then(|memory| memory.into_memory())
                else {
                    anyhow::bail!("No memory export");
                };

                let (memory, context) = memory.data_and_store_mut(&mut caller);
                let memory = read_slice_mut(memory, len, ptr, "data")?;

                let data_len = context.data.len();
                if data_len > memory.len() {
                    log_call!(context, "(reso.data len:{len} ptr:{ptr}) → {data_len}");
                    return Ok(data_len as i32);
                }

                let memory = &mut memory[..data_len];
                memory.copy_from_slice(context.data.as_bytes());

                log_call!(context, "(reso.data len:{len} ptr:{ptr}) → {data_len}");
                Ok(data_len as i32)
            },
        )
        .unwrap();

    // reso.previous_data – same as reso.data but with the previous data instead.
    linker
        .func_wrap(
            "reso",
            "previous_data",
            |mut caller: wasmtime::Caller<'_, Context>,
             len: i32,
             ptr: i32|
             -> wasmtime::Result<i32> {
                let Some(memory) = caller
                    .get_export("memory")
                    .and_then(|memory| memory.into_memory())
                else {
                    anyhow::bail!("No memory export");
                };

                let (memory, context) = memory.data_and_store_mut(&mut caller);
                let memory = read_slice_mut(memory, len, ptr, "previous_data")?;

                let previous_data_len = context.previous_data.len();
                if previous_data_len > memory.len() {
                    log_call!(
                        context,
                        "(reso.previous_data len:{len} ptr:{ptr}) → {previous_data_len}"
                    );
                    return Ok(previous_data_len as i32);
                }

                let memory = &mut memory[..previous_data_len];
                memory.copy_from_slice(context.previous_data.as_bytes());

                log_call!(
                    context,
                    "(reso.previous_data len:{len} ptr:{ptr}) → {previous_data_len}"
                );
                Ok(previous_data_len as i32)
            },
        )
        .unwrap();

    // reso.error – the field (as specified as a UTF-8 string of length `field_len` that starts in
    // memory at `field_ptr`) is invalid. The reason is provided in the message (as specified as a
    // UTF-8 string of length `message_len` that starts in memory at `message_ptr`).
    linker.func_wrap(
        "reso",
        "error",
        |mut caller: wasmtime::Caller<'_, Context>,
         field_len: i32,
         field_ptr: i32,
         message_len: i32,
         message_ptr: i32|
         -> wasmtime::Result<()> {
            let Some(memory) = caller
                .get_export("memory")
                .and_then(|memory| memory.into_memory())
            else {
                anyhow::bail!("No memory export");
            };

            let memory = memory.data(&caller);
            let context = caller.data();

            let field = read_string(memory, field_len, field_ptr, "field")?;
            let message = read_string(memory, message_len, message_ptr, "message")?;

            log_call!(
                context,
                "(reso.error field_len:{field_len} field_ptr:{field_ptr} message_len:{message_len} message_ptr:{message_ptr})"
            );
            println!("❗️ {field}: {message}");

            Ok(())
        },
    ).unwrap();

    // reso.warn – the field has a warning. The reason is provided in the message.
    linker.func_wrap(
        "reso",
        "warn",
        |mut caller: wasmtime::Caller<'_, Context>,
         field_len: i32,
         field_ptr: i32,
         message_len: i32,
         message_ptr: i32|
         -> wasmtime::Result<()> {
            let Some(memory) = caller
                .get_export("memory")
                .and_then(|memory| memory.into_memory())
            else {
                anyhow::bail!("No memory export");
            };

            let memory = memory.data(&caller);
            let context = caller.data();

            let field = read_string(memory, field_len, field_ptr, "field")?;
            let message = read_string(memory, message_len, message_ptr, "message")?;

            log_call!(
                context,
                "(reso.warn field_len:{field_len} field_ptr:{field_ptr} message_len:{message_len} message_ptr:{message_ptr})"
            );
            println!("⚠️ {field}: {message}");

            Ok(())
        },
    ).unwrap();

    // reso.diagnostic – a way for modules to output information. Takes a single string (represented
    // by a len+address pair).
    linker
        .func_wrap(
            "reso",
            "diagnostic",
            |mut caller: wasmtime::Caller<'_, Context>,
             len: i32,
             ptr: i32|
             -> wasmtime::Result<()> {
                let Some(memory) = caller
                    .get_export("memory")
                    .and_then(|memory| memory.into_memory())
                else {
                    anyhow::bail!("No memory export");
                };

                let memory = memory.data(&caller);
                let context = caller.data();

                log_call!(context, "(reso.diagnostic len:{len} ptr:{ptr})");
                let diagnostic = read_string_lax(memory, len, ptr, "diagnostic")?;

                // log_call!(context, "(reso.diagnostic len:{len} ptr:{ptr})");
                if context.verbose > 0 {
                    println!("ℹ️  {diagnostic}");
                }

                Ok(())
            },
        )
        .unwrap();

    // reso.set_required – set whether the field (len+address) is required (0 is not required, any
    // other value is required)
    linker
        .func_wrap(
            "reso",
            "set_required",
            |mut caller: wasmtime::Caller<'_, Context>,
             len: i32,
             ptr: i32,
             value: i32|
             -> wasmtime::Result<()> {
                let Some(memory) = caller
                    .get_export("memory")
                    .and_then(|memory| memory.into_memory())
                else {
                    anyhow::bail!("No memory export");
                };

                let memory = memory.data(&caller);
                let context = caller.data();

                let field = read_string(memory, len, ptr, "field")?;

                log_call!(
                    context,
                    "(reso.set_required len:{len} ptr:{ptr} value:{value})"
                );
                println!(
                    "💬 {field} is \x1b[35m{}\x1b[0m",
                    if value == 0 {
                        "not required"
                    } else {
                        "required"
                    }
                );

                Ok(())
            },
        )
        .unwrap();

    // reso.set_display – set whether the field (len+address) should be displayed (0 is do not
    // display, any other value is yes display the field)
    linker
        .func_wrap(
            "reso",
            "set_display",
            |mut caller: wasmtime::Caller<'_, Context>,
             len: i32,
             ptr: i32,
             value: i32|
             -> wasmtime::Result<()> {
                let Some(memory) = caller
                    .get_export("memory")
                    .and_then(|memory| memory.into_memory())
                else {
                    anyhow::bail!("No memory export");
                };

                let memory = memory.data(&caller);
                let context = caller.data();

                let field = read_string(memory, len, ptr, "field")?;

                log_call!(
                    context,
                    "(reso.set_visible len:{len} ptr:{ptr} value:{value})"
                );
                println!(
                    "💬 {field} is \x1b[35m{}\x1b[0m",
                    if value == 0 { "not visible" } else { "visible" }
                );

                Ok(())
            },
        )
        .unwrap();

    // reso.set – set a field to the provided value. The field is provided as a len+addr pair. The
    // value is provided as a len+addr pair that is expected to be JSON data.
    linker
        .func_wrap(
            "reso",
            "set",
            |mut caller: wasmtime::Caller<'_, Context>,
             field_len: i32,
             field_ptr: i32,
             value_len: i32,
             value_ptr: i32|
             -> wasmtime::Result<()> {
                let Some(memory) = caller
                    .get_export("memory")
                    .and_then(|memory| memory.into_memory())
                else {
                    anyhow::bail!("No memory export");
                };

                let memory = memory.data(&caller);
                let context = caller.data();

                let field = read_string(memory, field_len, field_ptr, "field")?;
                let value = read_string(memory, value_len, value_ptr, "field")?;
                let Ok(value) = serde_json::from_str::<serde_json::Value>(value) else {
                    anyhow::bail!("value was not a valid JSON value");
                };

                log_call!(
                    context,
                    "(reso.set field_len:{field_len} field_ptr:{field_ptr} value_len:{value_len} value_ptr:{value_ptr})"
                );
                println!(
                    "✏️  {field} set to \x1b[36m{}\x1b[0m",
                    serde_json::to_string(&value).unwrap(),
                );

                Ok(())
            },
        )
        .unwrap();

    // Any other import is allowed, but won't do anything useful. This is required because some
    // languages implicitly assume that wasm is compiled as wasi, and provide imports for wasi, even
    // if the module never calls them.
    linker.define_unknown_imports_as_traps(module).unwrap();

    linker
}

/// Read a string from the WebAssembly module's memory
///
/// If it doesn't happen to be UTF-8, that's fine; do our best.
fn read_string_lax<'a>(
    memory: &'a [u8],
    len: i32,
    ptr: i32,
    name: &str,
) -> wasmtime::Result<Cow<'a, str>> {
    let slice = read_slice(memory, len, ptr, name)?;

    Ok(String::from_utf8_lossy(slice))
}

/// Read a string from the WebAssembly module's memory
///
/// Fail if it isn't UTF-8.
fn read_string<'a>(memory: &'a [u8], len: i32, ptr: i32, name: &str) -> wasmtime::Result<&'a str> {
    let slice = read_slice(memory, len, ptr, name)?;

    match std::str::from_utf8(slice) {
        Ok(str) => Ok(str),
        Err(_err) => anyhow::bail!("{name} is invalid UTF-8"),
    }
}

/// Read a slice from a WebAssembly module's memory
///
/// Fail if the length or pointer are invalid.
fn read_slice<'a>(memory: &'a [u8], len: i32, ptr: i32, name: &str) -> wasmtime::Result<&'a [u8]> {
    if len < 0 {
        anyhow::bail!("{name} length is less than zero");
    }
    if ptr < 0 {
        anyhow::bail!("{name} pointer is less than zero");
    }
    let ptr = ptr as usize;
    let len = len as usize;

    let memory = &memory[ptr..];
    if memory.len() < len {
        anyhow::bail!("{name} length is invalid");
    }

    Ok(&memory[..len])
}

/// Read a slice from a WebAssembly module's memory
///
/// Fail if the length or pointer are invalid.
fn read_slice_mut<'a>(
    memory: &'a mut [u8],
    len: i32,
    ptr: i32,
    name: &str,
) -> wasmtime::Result<&'a mut [u8]> {
    if len < 0 {
        anyhow::bail!("{name} length is less than zero");
    }
    if ptr < 0 {
        anyhow::bail!("{name} pointer is less than zero");
    }
    let ptr = ptr as usize;
    let len = len as usize;

    let memory = &mut memory[ptr..];
    if memory.len() < len {
        anyhow::bail!("{name} length is invalid");
    }

    Ok(&mut memory[..len])
}
