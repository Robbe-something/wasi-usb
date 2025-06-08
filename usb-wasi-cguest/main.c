#include <stdio.h>
#include <stdbool.h>
#include <unistd.h>
#include "bindings/cguest.h"

bool exports_wasi_cli_run_run(void) {
    component_usb_device_libusb_error_t err;

    // Initialize the USB backend
    if (!component_usb_device_init(&err)) {
        fprintf(stderr, "Could not init backend: %d\n", err);
        return 1;
    }

    // Enable hot-plug
    if (!component_usb_usb_hotplug_enable_hotplug(&err)) {
        fprintf(stderr, "Hot-plug not available: %d\n", err);
        return 1;
    }
    printf("Hot-plug enabled – attach or remove a USB device to test.\n");

    // Poll for events
    for (int i = 0; i < 60; i++) {
        sleep(1);
        printf("Waiting for events...\n");

        component_usb_usb_hotplug_list_tuple3_event_info_own_usb_device_t events;
        component_usb_usb_hotplug_poll_events(&events);

        for (size_t j = 0; j < events.len; j++) {
            component_usb_usb_hotplug_tuple3_event_info_own_usb_device_t event_info = events.ptr[j];
            if (event_info.f0 & COMPONENT_USB_USB_HOTPLUG_EVENT_ARRIVED) {
                printf("ARRIVED bus %03d addr %03d %04x:%04x\n",
                       event_info.f1.bus, event_info.f1.address,
                       event_info.f1.vendor, event_info.f1.product);

                // Open the device
                component_usb_device_own_device_handle_t device_handle;
                if (!component_usb_device_method_usb_device_open(
                        component_usb_device_borrow_usb_device(event_info.f2), &device_handle, &err)) {
                    fprintf(stderr, "Failed to open device: %d\n", err);
                    continue;
                }

                // Claim interface 0
                if (!component_usb_device_method_device_handle_claim_interface(
                        component_usb_device_borrow_device_handle(device_handle), 0, &err)) {
                    fprintf(stderr, "Failed to claim interface: %d\n", err);
                    component_usb_device_method_device_handle_close(
                        component_usb_device_borrow_device_handle(device_handle));
                    component_usb_device_device_handle_drop_own(device_handle);
                    continue;
                }

                // Perform a transfer (example: control transfer)
                component_usb_device_transfer_setup_t setup = {
                    .bm_request_type = 0x80, // Direction: IN, Type: Standard, Recipient: Device
                    .b_request = 0x06,      // GET_DESCRIPTOR
                    .w_value = 0x0100,      // Descriptor Type (Device) and Index
                    .w_index = 0x0000       // Language ID
                };
                component_usb_device_transfer_options_t options = {
                    .endpoint = 0x00,       // Endpoint 0
                    .timeout_ms = 1000,     // 1 second timeout
                    .stream_id = 0,
                    .iso_packets = 0
                };
                component_usb_device_own_transfer_t transfer;
                if (!component_usb_device_method_device_handle_new_transfer(
                        component_usb_device_borrow_device_handle(device_handle),
                        COMPONENT_USB_TRANSFERS_TRANSFER_TYPE_CONTROL, &setup, 64, &options, &transfer, &err)) {
                    fprintf(stderr, "Failed to create transfer: %d\n", err);
                } else {
                    printf("Transfer created successfully.\n");

                    // Submit the transfer
                    cguest_list_u8_t data = { .ptr = NULL, .len = 0 };
                    if (!component_usb_transfers_method_transfer_submit_transfer(
                            component_usb_transfers_borrow_transfer(transfer), &data, &err)) {
                        fprintf(stderr, "Failed to submit transfer: %d\n", err);
                    } else {
                        printf("Transfer submitted successfully.\n");

                        // Await the transfer
                        cguest_list_u8_t result;
                        if (!component_usb_transfers_await_transfer(transfer, &result, &err)) {
                            fprintf(stderr, "Failed to await transfer: %d\n", err);
                        } else {
                            printf("Transfer completed successfully. Received %zu bytes.\n", result.len);
                            for (size_t i = 0; i < result.len; i++) {
                                printf("%02x ", result.ptr[i]);
                            }
                            printf("\n");
                            cguest_list_u8_free(&result);
                        }
                    }
                }

                // Release the interface and close the device
                component_usb_device_method_device_handle_release_interface(
                    component_usb_device_borrow_device_handle(device_handle), 0, &err);
                component_usb_device_method_device_handle_close(
                    component_usb_device_borrow_device_handle(device_handle));
                component_usb_device_device_handle_drop_own(device_handle);
            } else if (event_info.f0 & COMPONENT_USB_USB_HOTPLUG_EVENT_LEFT) {
                printf("LEFT    bus %03d addr %03d %04x:%04x\n",
                       event_info.f1.bus, event_info.f1.address,
                       event_info.f1.vendor, event_info.f1.product);
            }
        }

        component_usb_usb_hotplug_list_tuple3_event_info_own_usb_device_free(&events);
    }

    printf("Done – no more polling.\n");
    return true;
}