# usb-c-host

This is a test program to compile c to wasm and run it with the custom component.

## generating the bindings

```bash
# install wit-bindgen and wasm-tools
cargo install wit-bindgen-cli
cargo install --locked wasm-tools
# use wit-bindgen to generate the bindings
# wit-bindgen c ../wit --world cguest --out-dir bindings
```

## Build

### prepare the environment
```bash
# set the path to the wasi-sdk
export WASI_SDK_BIN=/path/to/wasi-sdk/bin

# download the correct version of the adapter
export WASMTIME_VERSION=v31.0.0
curl -L https://github.com/bytecodealliance/wasmtime/releases/download/${WASMTIME_VERSION}/wasi_snapshot_preview1.reactor.wasm -o helper/wasi_snapshot_preview1.reactor.wasm
```

### generate the wasm file
```bash
# build the c code
${WASI_SDK_BIN}/clang main.c bindings/cguest.c bindings/cguest_component_type.o --target=wasm32-wasip1 -o out/program.wasm -mexec-model=reactor
# adapt the generated wasm file to wasmtime
wasm-tools component new out/program.wasm \
  --adapt helper/wasi_snapshot_preview1.reactor.wasm \
  -o out/program.component.wasm
```

## Run
```bash
# run the program with the custom wasmtime runtime
../usb-wasi-host/target/debug/usb-wasi-host -c out/program.component.wasm
```