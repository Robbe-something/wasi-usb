# -------------------------------
# Simple USB-WASI example runner
# -------------------------------
# which example to build/run
EXAMPLE ?= smoke
# debug (default) | release
MODE    ?= debug

# --- derive cargo flags + output dir -------------------------------
ifeq ($(MODE),release)
    CARGO_MODE_FLAG := --release --target=wasm32-wasip2
    OUT_DIR        := release
    HOST_BUILD_FLAG := --release
else                              # debug
    CARGO_MODE_FLAG := --target=wasm32-wasip2
    OUT_DIR        := debug
    HOST_BUILD_FLAG :=
endif

WASM = target/wasm32-wasip2/$(OUT_DIR)/examples/$(EXAMPLE).wasm
HOST = ../usb-wasi-host/target/release/usb-wasi-host

# -------------------------------------------------------------------
build:
	cargo build --example $(EXAMPLE) $(CARGO_MODE_FLAG)
	cd ../usb-wasi-host && cargo build $(HOST_BUILD_FLAG)

run: build
	time sudo $(HOST) -c $(WASM) -l error

.PHONY: build run