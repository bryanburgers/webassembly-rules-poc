use clap::Parser;
use rcp19_to_wasm_common::Rules;
use std::path::PathBuf;

const WASM: &[u8] = include_bytes!("../template.wasm");

/// The struct that represents command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The rules file in the very specific rules format that https://rules.zenlist.dev uses
    #[arg(short, long, value_name = "FILE")]
    rules: PathBuf,

    /// The destination to write the wasm file to
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,
}

const WASM_PAGE_SIZE: u32 = 65536;

fn main() {
    let args = Args::parse();

    let rules_data = std::fs::read(args.rules).expect("Expected rules file to be readable");
    let rules: Rules =
        serde_json::from_slice(&rules_data).expect("Expected rules file to be valid JSON");
    validate_rules(&rules);
    let rules_data = serde_json::to_vec(&rules).unwrap();

    let mut module =
        walrus::Module::from_buffer(WASM).expect("Expected wasm blob to be valid wasm");

    // Add an extra memory page to the module so that, when we copy our data in, we don't overwrite
    // anything that the module previously needed.
    let memory = module
        .memories
        .iter_mut()
        .next()
        .expect("Expected wasm blob to have a memory");
    let previous_initial = memory.initial;
    memory.initial += 1;

    // The offset, in bytes, of where we are going to write the rules data.
    let data_offset = previous_initial * WASM_PAGE_SIZE;
    // The length of the rules data.
    let data_len = rules_data.len();

    // Create a new data section to hold the rules data.
    module.data.add(
        walrus::DataKind::Active(walrus::ActiveData {
            memory: memory.id(),
            location: walrus::ActiveDataLocation::Absolute(data_offset),
        }),
        rules_data,
    );

    // Find the exported function called `validate_target`. This is the function that behaves like
    // the spec's `validate`, but that takes the location and length of the rules it needs to
    // evaluate.
    let validate_target_export = module
        .exports
        .iter()
        .find(|export| {
            export.name == "validate_target"
                && matches!(export.item, walrus::ExportItem::Function(_))
        })
        .expect("Expected module to have an exported function named 'validate_target'");
    let validate_target_export_id = validate_target_export.id();
    let walrus::ExportItem::Function(validate_target_function_id) = validate_target_export.item
    else {
        panic!("Expected module to have an exported function named 'validate_target'");
    };

    // Create the actual `validate` function. This function is very simple: it calls
    // `validate_target` with the location and length of the rules data.
    let mut builder = walrus::FunctionBuilder::new(&mut module.types, &[], &[]);
    builder
        .func_body()
        .i32_const(data_offset as i32)
        .i32_const(data_len as i32)
        .call(validate_target_function_id);
    let function_id = builder.finish(vec![], &mut module.funcs);

    // And export the newly created `validate` function.
    module.exports.add("validate", function_id);
    // Probably unnecessary, but stop exporting the old `validate_target` function, since it doesn't
    // need to be exported.
    module.exports.delete(validate_target_export_id);

    // And write out the wasm file!
    module
        .emit_wasm_file(args.output)
        .expect("Failed to output file");
}

fn validate_rules(rules: &Rules) {
    for rule in &rules.value {
        rule.rule_expression
            .parse::<rets_expression::Expression>()
            .expect("Failed to parse rule");
    }
}
