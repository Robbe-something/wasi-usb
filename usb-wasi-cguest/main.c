#include <stdio.h>

#include "bindings/cguest.h"

bool exports_wasi_cli_run_run(void) {
    component_usb_device_libusb_error_t err;
    bool ok = component_usb_device_init(&err);

    if (!ok) {
        fprintf(stderr, "Failed to initialize USB device: %d\n", err);
        return false;
    }

    printf("Initialized USB device\n");

    return true;
}
