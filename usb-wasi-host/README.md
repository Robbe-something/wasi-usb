# usb-wasi-host

This folder contains the complete implementation of the WASI-USB interface, by modifying the wasmtime runtime.

## building

A release version of the runtime can be built by running the following command in this folder:
```bash
cargo build --release
```

## using

The created executable can be run with different flags as parameters:

```
Usage: usb-wasi-host [OPTIONS] --component-path <COMPONENT_PATH>

Options:
  -c, --component-path <COMPONENT_PATH>
  -d, --usb-devices <USB_DEVICES>
  -u, --use-allow-list
  -l, --debug_level <DEBUG_LEVEL>        [default: info]
  -h, --help                             Print help
```