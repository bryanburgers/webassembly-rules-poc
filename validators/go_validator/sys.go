package main;

import (
	"unsafe";
	"encoding/json";
);

func data(v any) {
	// Ask the host how large the JSON is
	dataLen := sys_data(0, unsafe.Pointer(uintptr(0)));
	// Make a buffer that large
	byteArray := make([]byte, dataLen);
	// And ask the host to fill our buffer with the JSON
	sys_data(dataLen, unsafe.Pointer(unsafe.SliceData(byteArray)));

	// Now try to unmarshal the JSON.
	err := json.Unmarshal(byteArray, v)
	if err != nil {
		diagnostic("Unmarshaling data failed");
		panic("");
	}
}

func previousData(v any) {
	// Ask the host how large the JSON is
	dataLen := sys_previous_data(0, unsafe.Pointer(uintptr(0)));
	// Make a buffer that large
	byteArray := make([]byte, dataLen);
	// And ask the host to fill our buffer with the JSON
	sys_previous_data(dataLen, unsafe.Pointer(unsafe.SliceData(byteArray)));

	// Now try to unmarshal the JSON.
	err := json.Unmarshal(byteArray, v)
	if err != nil {
		diagnostic("Unmarshaling previousData failed");
		panic("");
	}
}

func diagnostic(str string) {
	var l int32;
	l = int32(len(str));
	p := unsafe.Pointer(unsafe.StringData(str));
	sys_diagnostic(l, p);
}

func error(field string, message string) {
	field_len := int32(len(field))
	field_ptr := unsafe.Pointer(unsafe.StringData(field));
	message_len := int32(len(message))
	message_ptr := unsafe.Pointer(unsafe.StringData(message));
	sys_error(field_len, field_ptr, message_len, message_ptr);
}

func set_required(field string, value bool) {
	field_len := int32(len(field))
	field_ptr := unsafe.Pointer(unsafe.StringData(field));
	required := 0;
	if value {
		required = 1
	}
	sys_set_required(field_len, field_ptr, int32(required));
}

func set_display(field string, value bool) {
	field_len := int32(len(field))
	field_ptr := unsafe.Pointer(unsafe.StringData(field));
	display := 0;
	if value {
		display = 1
	}
	sys_set_display(field_len, field_ptr, int32(display));
}

func set(field string, value any) {
	field_len := int32(len(field))
	field_ptr := unsafe.Pointer(unsafe.StringData(field));
	buf, err := json.Marshal(value)
	if err != nil {
		diagnostic("Failed to set");
		panic("");
	}
	data_len := int32(len(buf));
	data_ptr := unsafe.Pointer(unsafe.SliceData(buf));
	sys_set(field_len, field_ptr, data_len, data_ptr);
}

//go:wasmimport reso diagnostic
func sys_diagnostic(len int32, ptr unsafe.Pointer);
//go:wasmimport reso data
func sys_data(len int32, ptr unsafe.Pointer) int32;
//go:wasmimport reso previous_data
func sys_previous_data(len int32, ptr unsafe.Pointer) int32;
//go:wasmimport reso error
func sys_error(field_len int32, field_ptr unsafe.Pointer, message_len int32, message_ptr unsafe.Pointer);
//go:wasmimport reso set_required
func sys_set_required(field_len int32, field_ptr unsafe.Pointer, value int32);
//go:wasmimport reso set_display
func sys_set_display(field_len int32, field_ptr unsafe.Pointer, value int32);
//go:wasmimport reso set
func sys_set(field_len int32, field_ptr unsafe.Pointer, value_len int32, value_ptr unsafe.Pointer);