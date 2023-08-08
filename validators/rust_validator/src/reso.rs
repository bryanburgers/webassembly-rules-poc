#![allow(dead_code)]

use serde::{de::DeserializeOwned, Serialize};

pub fn data<T>() -> T
where
    T: DeserializeOwned,
{
    let mut data = [0_u8; 10 * 1024];
    let ptr = data.as_mut_ptr();
    let len = data.len();
    let size = unsafe { sys::data(len as i32, ptr as i32) } as usize;
    if size > len {
        diagnostic("data is longer than available space");
        panic!();
    }
    match serde_json::from_slice(&data[..size]) {
        Ok(value) => value,
        Err(_) => {
            diagnostic("data was not valid JSON");
            panic!();
        }
    }
}

pub fn previous_data<T>() -> T
where
    T: DeserializeOwned,
{
    let mut data = [0_u8; 10 * 1024];
    let ptr = data.as_mut_ptr();
    let len = data.len();
    let size = unsafe { sys::data(len as i32, ptr as i32) } as usize;
    if size > len {
        diagnostic("previous_data is longer than available space");
        panic!();
    }
    match serde_json::from_slice(&data[..size]) {
        Ok(value) => value,
        Err(_) => {
            diagnostic("previous_data was not valid JSON");
            panic!();
        }
    }
}

pub fn error(field: &str, message: &str) {
    let field_len = field.len();
    let field_ptr = field.as_ptr();
    let message_len = message.len();
    let message_ptr = message.as_ptr();
    unsafe {
        sys::error(
            field_len as i32,
            field_ptr as i32,
            message_len as i32,
            message_ptr as i32,
        );
    }
}

pub fn warn(field: &str, message: &str) {
    let field_len = field.len();
    let field_ptr = field.as_ptr();
    let message_len = message.len();
    let message_ptr = message.as_ptr();
    unsafe {
        sys::warn(
            field_len as i32,
            field_ptr as i32,
            message_len as i32,
            message_ptr as i32,
        );
    }
}

pub fn diagnostic(message: &str) {
    let len = message.len();
    let ptr = message.as_ptr();
    unsafe {
        sys::diagnostic(len as i32, ptr as i32);
    }
}

pub fn set<T>(field: &str, value: T)
where
    T: Serialize,
{
    let value_string = serde_json::to_string(&value).unwrap();
    let field_len = field.len();
    let field_ptr = field.as_ptr();
    let value_len = value_string.len();
    let value_ptr = value_string.as_ptr();
    unsafe {
        sys::set(
            field_len as i32,
            field_ptr as i32,
            value_len as i32,
            value_ptr as i32,
        );
    }
}

pub fn set_required(field: &str, required: bool) {
    let field_len = field.len();
    let field_ptr = field.as_ptr();
    unsafe {
        sys::set_required(
            field_len as i32,
            field_ptr as i32,
            if required { 1 } else { 0 },
        );
    }
}

pub fn set_display(field: &str, display: bool) {
    let field_len = field.len();
    let field_ptr = field.as_ptr();
    unsafe {
        sys::set_display(
            field_len as i32,
            field_ptr as i32,
            if display { 1 } else { 0 },
        );
    }
}

mod sys {
    #[link(wasm_import_module = "reso")]
    extern "C" {
        pub fn error(field_len: i32, field_ptr: i32, message_len: i32, message_ptr: i32);
        pub fn warn(field_len: i32, field_ptr: i32, message_len: i32, message_ptr: i32);
        pub fn diagnostic(len: i32, ptr: i32);
        pub fn set_required(field_len: i32, field_ptr: i32, value: i32);
        pub fn set_display(field_len: i32, field_ptr: i32, value: i32);
        pub fn set(field_len: i32, field_ptr: i32, value_len: i32, value_ptr: i32);
        pub fn data(len: i32, ptr: i32) -> i32;
        pub fn previous_data(len: i32, ptr: i32) -> i32;
    }
}
