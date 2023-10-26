# rcp19-to-wasm

A tool that takes rules in RCP-19 format and generates a wasm binary blob
conforming to the [WebAssembly rules evaluator proposal][proposal].

## Use

`cargo run -p rcp19-to-wasm --rules input.json --output output.wasm`

This takes in an `input.json` that represents the rules, in the format used by 
[rules.zenlist.dev](https://rules.zenlist.dev) – specifically the format that
gets saved to the gist when sharing rules via that website.

The output is a wasm binary blob that can be run by the rules evaluator
proof-of-concept.

## Design

1. The project `rcp19-to-wasm-template` generates a wasm module that *almost*
   conforms to the wasm rules proposal. This project knows how to evaluate rules
   but does not know which rules it needs to evaluate. Instead of exporting a
   parameterless `validate` function according to the proposed spec, it exports
   a `validate_target` that accepts rules and then does the validate.
2. The `rcp19-to-wasm` modifies the wasm created by `rcp19-to-wasm-template`. It
   adds a data segment containing the rules, then adds a `validate` function
   that calls `validate_target` with the now-known location of the rules.

## Updating `rcp19-to-wasm-template`

First, run

```
cargo build -p rcp19-to-wasm-template --release --target wasm32-unknown-unknown
```

Then copy the module to its destination so that `rcp19-to-wasm` can use it.

```
cp target/wasm32-unknown-unknown/release/rcp19_to_wasm_template.wasm rcp19-to-wasm/template.wasm
```

[proposal]: https://github.com/RESOStandards/transport/discussions/92
