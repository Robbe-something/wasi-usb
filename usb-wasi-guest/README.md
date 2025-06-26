# usb-wasi-guest

This folder contains Rust examples using the WASI-USB interface defined in `/wit`.

## building
these examples can be build manually using `cargo build --release --target=wasm32-wasip2` into a wasm component, which can then be run by any runtime that implements the WASI-USB interface.

It is also possible to run `make run EXAMPLE={example_name}` in this folder, which would build both the example, the runtime and execute the example program on the runtime from usb-wasi-host.