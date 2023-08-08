## Summary

A proposal to use the WebAssembly standard as the basis for evaluating business rules.

This would supersede RCP19 and our use of RETS Validation Expressions.

## Introduction

Frequently in the lifecycle of a listing, it is valuable to determine if the current data that represents that listing is valid according to an MLS’ unique business rules.

There are typically two approaches to determining this validity:

- bring the data to the code
- bring the code to the data

From a frontend perspective, “bring the data to the code” typically works by sending the data to a validate endpoint (currently being specified) or publishing the listing and seeing if it was valid.

On the other hand, “bring the code to the data” allows the frontend to validate a listing in real time and provide an agent with immediate feedback. Currently this happens using RCP19 and RETS VE Expressions.

In both situations, validation is essentially a deterministic function from a few inputs to a list of outputs:

- Given a limited set of inputs
  - Current listing data
  - Previous listing data
  - (Server may use more inputs as it sees fit)
- Provide a limited set of outputs
  - Which fields are invalid and why
  - Which fields have warnings and what those warnings are
  - How should the UI change

This proposal deals with the “bring the code to the data” aspect of validation, and proposes that we bring that code to the data in the form of the widely used and standardized web assembly language.

To achieve this, we would specify a handful of host functions that the host must provide, a single function that the validation webassembly module must provide, and a standard interpretation of the output of the execution.

## Why WebAssembly?

Evaluating rules is essentially asking a frontend or an MLS System to efficiently run untrusted code from the MLS.

WebAssembly, with its origin as an efficient compile target for the on web, is designed specifically for this use case: efficiently running untrusted code in a sandbox.

WebAssembly started in the browser, but is now widely used in many applications – including on the server – where its sandboxed execution model is a benefit.

## Examples

Here are two examples of how this could work, written in Go (compiled using tinygo), and AssemblyScript (a Typescript-like language compiles easily to wasm) compiled to WebAssembly.

### Go

```go
package main;

func main() {}

//export validate
func validate() {
	var listing, previousListing Listing;
	data(&listing);
	previousData(&previousListing);

	if listing.ListPrice <= 0 {
		error("ListPrice", "List Price must be greater than $0");
	}
	if listing.MlsStatus == "Closed" {
		set_required("ClosePrice", true);
		set_display("ClosePrice", true);
	} else {
		set_required("ClosePrice", false);
		set_display("ClosePrice", false);
		set("ClosePrice", nil);
	}
}

type Listing struct {
    ListPrice int64
    MlsStatus string
}

```

### AssemblyScript

```typescript
import { JSON } from "assemblyscript-json/assembly";
import * as reso from "./reso";

export function validate(): void {
  const data = reso.data();
  const previousData = reso.previousData();

  if (!data.isObj) {
    reso.diagnostic("Data was not an object");
    unreachable();
  }

  const obj = <JSON.Obj>data;

  const listPriceOrNull = obj.getFloat("ListPrice");
  let listPrice = 0.0;
  if (listPriceOrNull !== null) {
    listPrice = listPriceOrNull.valueOf();
  }

  const mlsStatusOrNull = obj.getString("MlsStatus");
  let mlsStatus: string | null = null;
  if (mlsStatusOrNull !== null) {
    mlsStatus = mlsStatusOrNull.valueOf();
  }

  if (listPrice <= 0.0) {
    reso.error("ListPrice", "List price must be greater than $0");
  }

  if (mlsStatus === "Closed") {
    reso.setRequired("ClosePrice", true);
    reso.setDisplay("ClosePrice", true);
  } else {
    reso.setRequired("ClosePrice", false);
    reso.setDisplay("ClosePrice", false);
    reso.set("ClosePrice", new JSON.Null());
  }
}
```

A proof-of-concept of this proposal is available here: https://github.com/bryanburgers/webassembly-rules-poc

## Advantages & Disadvantages

Every technical decision comes with trade offs. While it is my opinion that this proposal has enough positive merits to be considered as a specification, I acknowledge that there are aspects of it that are worse than our current system for rules.

### Advantage: innovation without waiting on RESO or Frontends

Our current rules approach is powerful, but only allows MLSes to express rules that fit into our limited grammar.

Adding new functionality to our grammar to support new innovation takes time, both in terms of waiting for the specification to be updated and for frontends to implement the updates.

This WebAssembly-based approach to validation would allow MLSes to innovate using rules that are expressible in WASM without needing to wait on the standards body or clients to add support.

This allows validating on things we know about but the RETS VE spec don’t currently support (for example, a rule that a listing can’t be published if any media is smaller than 640x480, because our spec doesn’t support a scope resolution operator) and things we don’t know about (logical paradox to come up with one, but take for example “ListPrice must not be prime”).

### Advantage: WebAssembly has vibrant development

At any given time, there are only a handful of engineers that can work on RESO Rules and our custom grammar.

On the other hand, the WebAssembly ecosystem has tons of effort constantly being poured into it.

### Advantage: dynamic error messages

The current rules spec allows the MLS to define static error messages for each rule. This could be something like

> Public remarks must not include a phone number

This proposal allows for dynamic error messages, so an error could look something like

> Public remarks must not include a phone number, but 867-5309 looks like a phone number

This provides feedback to the agent that is even more actionable.

### Disadvantage: Non-portable rules

An advantage of using RCP19 rules is that the syntax of the rules themselves are portable. This would mean that, if many MLS Systems supported the same rule syntax, an MLS would be able to switch providers without needing to rewrite their rules.

This spec does not define rules in a portable way. This allows one MLS System to define how they do rules differently than another, which could encourage lock-in.

### Disadvantage: no way to subset rules

In some situations, frontends only deal with a subset of fields. In those situations there may be a desire to only get a subset of the rules that apply to these fields.

Because the WebAssembly module represents all of the rules for an MLS, this wouldn’t be possible.

### Disadvantage: size of WebAssembly modules

Many WebAssembly modules measure in the megabytes. That can be a rather large payload depending on how often it is transferred.

Additionally, the entity that is able to reduce this size (the MLS or the MLS System) and the entity that would prefer a smaller size (Frontend) may not be the same entity, so the incentives to reduce the module size may not be aligned.

### Disadvantage: opacity

A WebAssembly module is basically an opaque binary blob. While decompilation into WebAssembly Text Format (wat) is possible, it would not, in general, help anybody make sense of an MLS’ rules.

Note that while this is a disadvantage compared to using RETS VE, validating via a n MLS-provided validate API or validating while publishing is also submitting data to be verified by opaque rules, so this isn’t necessarily _worse_.

## API

### Conceptual API

Conceptually, the WebAssembly payload must implement exactly one function that the host can call:

`validate(data: JSON, previousData: JSON): void`

This function takes the current data and the previous data, calls zero or more functions providing feedback (errors, warnings, UI changes), and then returns nothing.

While validating, the function may call any of the following functions to provide feedback:

- `error(fieldName: string, message: string)` – the provided field has an error, described by the provided message; this is equivalent to the result of a REJECT or ACCEPT action
- `warning(fieldName: string, message: string)` – the provided field has a warning, described by the provided message; this is equivalent to the result of a WARN action
- `set(fieldName: string, value: JSON)` – set the value of the provided field to the provided value
- `set_required(fieldName: string, value: boolean, message: string)` – set whether a field is required
- `set_visible/set_readonly(fieldName: string, value: boolean)` – set whether a field is visible/read only
- `set_picklist(fieldName: string, values: string[])` – set the values for a picklist.
- `diagnostic(output: string)` – wasm modules have no way to talk to the outside world except through what is provided to them. That means they can’t log information anywhere. Providing a diagnostic function allows the module to output debugging info that the host can store or discard

### Actual API

WebAssembly currently has no concept of strings, so an API that uses strings is actually not possible.

The common way to pass strings out of wasm is for the module to store a string in its local memory, and then perform a host call with the address and length of that string. Because the host has access to the memory, it can pull out the string from memory directly.

The common way to pass strings into wasm is for the module to allocate space in its own local memory, and then call a host function to fill that memory space with data that represents a string.

In both situations, the host and the module need to agree on the format of the string. In this case, we specify that the string must be in UTF-8 format.

- `data(max_length: i32, address: i32): i32` – get the JSON blob that represents the current listing, as a UTF-8 string. The module requests that the host place the data (stringified JSON) into the module’s memory at the provided address. If successful, the return value is the number of bytes written (the length of the data). If there is not enough space to write the entire data, the host should not write any data and instead the return value is the number of bytes required. (This allows the module to create more space – exactly enough space – in its memory and call `data` a second time.) The module is responsible for turning this string into JSON for processing.
- `previous_data(max_length: i32, address: i32): i32` – similar to `data`, but for the previous data
- `current_timestamp(max_length: i32, address: i32)` – similar to `data` and `previous_data`, write an RFC3339-formatted timestamp into a buffer provided by the module. WebAssembly modules have no way to access any host information unless the host provides it; that includes the current time.
- `current_date(max_length: i32, address: i32)` – similar to `current_timestamp`, write an ISO-8601 date (YYYY-MM-DD) string into a buffer provided by the module.
- `error(field_len: i32, field_address: i32, message_len: i32, message_address: i32)` – the provided field has an error, described by the provided message (where both the field and message are a length+address pair)
- `warn(field_len: i32, field_address: i32, message_len: i32, message_address: i32)` – the provided field has a warning, described by the provided message (where both the field and message are a length+address pair)
- `set(field_len: i32, field_address: i32, value_len: i32, value_address: i32)` - set the provided field to the provided value. The value is expected to be the textual representation of a JSON value.
- `set_required(field_len: i32, field_address: i32, message_len: i32, message_address: i32, value: i32)` – set whether a field is required. The field is not required if value is zero; any other value means the field is required.
- `set_visible/set_readonly(field_len: i32, field_address: i32, value: i32)` – set whether a field is visible/readonly. The field is not visible/readonly if value is zero; any other value means the field is visible/readonly.
- `set_picklist` – TODO
- `diagnostic(len: i32, address: i32)` – Send diagnostic information to the host. This would typically not be visible to the end user.

## Adherence to requirements

A set of requirements for a rules grammar was set out in [where]. How does this proposal stack up to these requirements?

### MUST be human friendly.

❌ This isn’t. However, what the MLS does to produce the validation WebAssembly module still can be.

Already some MLSes define their rules in their own language, and transpile to RETS VE. This would be similar to that approach, only the target would not be human friendly.

### MUST be machine executable and platform independent.

✅ Yes. There are a large number of wasm runtimes that can be embedded into a large number of host languages on all major platforms.

WebAssembly runs in the browser (its original reason for existence) on all major browsers.

Other major implementations include:

- wasmtime
- wasmer
- https://github.com/wasm3/wasm3
- https://github.com/oracle/graal/tree/master/wasm
- https://github.com/appcypher/awesome-wasm-runtimes

### MUST support arithmetic, logical, and comparison operators.

✅ Yes. Anything that can be done in any language that compiles to WebAssembly can be done.

### MUST not be Turing Complete.

❌ No. WebAssembly is Turing complete.

It is my impression that this requirement exists for two reasons.

The first is that rule validation must not arbitrarily use all of the resources of a computer while running. WebAssembly is being used on the server by cloud providers who want the same protections, and as such WebAssembly runtimes have ways to limit compute time, memory usage, etc. WebAssembly in the browser can be cancelled (if run via a worker) after a defined amount of time.

The second is that the entity running the rules may not be the same entity as the entity executing the rules, so the entity running the rules may want assurances that the execution isn’t malicious or can’t mess with its own operation. WebAssembly has been designed, from the start, as an entirely sandboxed system that has no way to reach out of its sandbox except through the APIs provided.

### MUST support mutation (such as SET and SET_REQUIRED in RETS VE).

✅ Yes. See the set, set_required APIs.

### MUST use a context-free, LL grammar to ensure parsing always terminates. (RETS VE is LL(1)).

✅ I do not know exactly what form of grammar web assembly uses. However, large amount of use in the real world makes me unconcerned even if it does not fit this exact description.

### SHOULD use existing existing open rules standards, if possible.

✅ Yes. A large part of the reason for using webassembly is because it is a very successful standard.

### SHOULD have function parity with RETS VE (contains, substring, etc.) since the items that were in the spec captured the business needs at the time.

✅ Yes. Any rules would be runnable in wasm. For migration, we would want to provide some way to either compile RETS VE Rules to Webassembly, or to compile an evaluator to a web assembly module and an easy way to embed rules in that module, to achieve parity for existing RETS VE rules. I have no doubt that this would be possible.
