# -------------------------------
# Simple USB-WASI example runner
# -------------------------------
# which example to build/run
EXAMPLE ?= smoke
# debug (default) | release
MODE    ?= debug

# --- derive cargo flags + output dir -------------------------------
ifeq ($(MODE),release)
    CARGO_MODE_FLAG := --target=wam32-wasip2 --release
    OUT_DIR        := release
    HOST_BUILD_FLAG := --release
else                              # debug
    CARGO_MODE_FLAG := --target=wasm32-wasip2
    OUT_DIR        := debug
    HOST_BUILD_FLAG :=
endif

WASM = target/wasm32-wasip2/$(OUT_DIR)/examples/$(EXAMPLE).wasm
HOST = ../usb-wasi-host/target/$(MODE)/usb-wasi-host

# -------------------------------------------------------------------
build:
	cargo build --example $(EXAMPLE) $(CARGO_MODE_FLAG)
	cd ../usb-wasi-host && cargo build $(HOST_BUILD_FLAG)

run: build
	time sudo $(HOST) -c $(WASM) --usb-devices 05ac:139d --usb-devices 05ac:12a8 -d 0951:1666 -d 046d:c52b -l trace

.PHONY: build run