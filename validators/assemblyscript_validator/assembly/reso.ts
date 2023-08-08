import { JSON } from "assemblyscript-json/assembly"; 

export function diagnostic(s: string): void {
    const arrayBuffer = String.UTF8.encode(s, false);
    sys_diagnostic(arrayBuffer.byteLength, changetype<i32>(arrayBuffer));
}

export function data(): JSON.Value {
    // Ask the host for the length of the JSON
    const len = sys_data(0, 0);
    // Create a new buffer for the JSON
    const buffer = new ArrayBuffer(len);
    // Ask the host to fill the buffer
    sys_data(len, changetype<i32>(buffer));

    // Convert the UTF8 data into a string
    const string = String.UTF8.decode(buffer, false);

    // And parse that string
    return JSON.parse(string);
}

export function previousData(): JSON.Value {
    // Ask the host for the length of the JSON
    const len = sys_previous_data(0, 0);
    // Create a new buffer for the JSON
    const buffer = new ArrayBuffer(len);
    // Ask the host to fill the buffer
    sys_previous_data(len, changetype<i32>(buffer));

    // Convert the UTF8 data into a string
    const string = String.UTF8.decode(buffer, false);

    // And parse that string
    return JSON.parse(string);
}

export function error(field: string, message: string): void {
    const fieldBuffer = String.UTF8.encode(field, false);
    const messageBuffer = String.UTF8.encode(message, false);
    sys_error(fieldBuffer.byteLength, changetype<i32>(fieldBuffer), messageBuffer.byteLength, changetype<i32>(messageBuffer));
}

export function warn(field: string, message: string): void {
    const fieldBuffer = String.UTF8.encode(field, false);
    const messageBuffer = String.UTF8.encode(message, false);
    sys_warn(fieldBuffer.byteLength, changetype<i32>(fieldBuffer), messageBuffer.byteLength, changetype<i32>(messageBuffer));
}

export function setRequired(field: string, value: boolean): void {
    const fieldBuffer = String.UTF8.encode(field, false);
    sys_set_required(fieldBuffer.byteLength, changetype<i32>(fieldBuffer), value ? 1 : 0);
}

export function setDisplay(field: string, value: boolean): void {
    const fieldBuffer = String.UTF8.encode(field, false);
    sys_set_display(fieldBuffer.byteLength, changetype<i32>(fieldBuffer), value ? 1 : 0);
}

export function set(field: string, value: JSON.Value): void {
    const fieldBuffer = String.UTF8.encode(field, false);
    const valueBuffer = String.UTF8.encode(value.stringify(), false);
    sys_set(fieldBuffer.byteLength, changetype<i32>(fieldBuffer), valueBuffer.byteLength, changetype<i32>(valueBuffer));
}

@external("reso", "data")
declare function sys_data(len: i32, ptr: i32): i32;

@external("reso", "previous_data")
declare function sys_previous_data(len: i32, ptr: i32): i32;

@external("reso", "diagnostic")
declare function sys_diagnostic(len: i32, ptr: i32): void;

@external("reso", "error")
declare function sys_error(field_len: i32, field_ptr: i32, message_len: i32, message_ptr: i32): void;

@external("reso", "warn")
declare function sys_warn(field_len: i32, field_ptr: i32, message_len: i32, message_ptr: i32): void;

@external("reso", "set_required")
declare function sys_set_required(len: i32, ptr: i32, value: i32): void;

@external("reso", "set_display")
declare function sys_set_display(len: i32, ptr: i32, value: i32): void;

@external("reso", "set")
declare function sys_set(field_len: i32, field_ptr: i32, value_len: i32, value_ptr: i32): void;