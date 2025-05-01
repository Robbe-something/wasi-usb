// wasm_backend.c

#include <libusb.h>
#include "libusbi.h"

// Forward declarations of the WIT-imported functions (stubs).
// These will be implemented in the WASM backend component.

// Initialization & cleanup
extern int wasm_init(struct libusb_context *ctx);
extern void wasm_exit(struct libusb_context *ctx);
extern int wasm_set_option(struct libusb_context *ctx, enum libusb_option option, va_list value);

// Device enumeration & management
extern int wasm_get_device_list(struct libusb_context *ctx, struct discovered_devs **discdevs);
extern void wasm_hotplug_poll(void);
extern int wasm_wrap_sys_device(struct libusb_device *dev, struct libusb_device_handle *handle, intptr_t sys_dev);
extern int wasm_open_device(struct libusb_device_handle *handle);
extern void wasm_close_device(struct libusb_device_handle *handle);
extern void wasm_destroy_device(struct libusb_device *dev);

// Descriptor retrieval
//extern int wasm_get_device_descriptor(struct libusb_device *dev, unsigned char *buffer, int *host_endian);
extern int wasm_get_active_config_descriptor(struct libusb_device *dev, unsigned char *buffer, size_t len, int *host_endian);
extern int wasm_get_config_descriptor(struct libusb_device *dev, uint8_t config_index, unsigned char *buffer, size_t len, int *host_endian);
extern int wasm_get_config_descriptor_by_value(struct libusb_device *dev, uint8_t bConfigurationValue, unsigned char **buffer, int *host_endian);

// Configuration & interface management
extern int wasm_get_configuration(struct libusb_device_handle *handle, int *config);
extern int wasm_set_configuration(struct libusb_device_handle *handle, int config);
extern int wasm_claim_interface(struct libusb_device_handle *handle, int interface_number);
extern int wasm_release_interface(struct libusb_device_handle *handle, int interface_number);
extern int wasm_set_interface_altsetting(struct libusb_device_handle *handle, int interface_number, int altsetting);
extern int wasm_clear_halt(struct libusb_device_handle *handle, unsigned char endpoint);
extern int wasm_reset_device(struct libusb_device_handle *handle);

// Memory management
extern int wasm_alloc_stream(struct libusb_device_handle *handle, uint32_t num_streams, unsigned char *endpoints, int num_endpoints);
extern int wasm_free_stream(struct libusb_device_handle *handle, unsigned char *endpoints, int num_endpoints);
extern void *wasm_dev_mem_alloc(struct libusb_device *dev, size_t len);
extern void wasm_dev_mem_free(struct libusb_device *dev, void *buffer, size_t len);

// Kernel driver interaction (optional)
extern int wasm_kernel_driver_active(struct libusb_device_handle *handle, int interface_number);
extern int wasm_detach_kernel_driver(struct libusb_device_handle *handle, int interface_number);
extern int wasm_attach_kernel_driver(struct libusb_device_handle *handle, int interface_number);

// Asynchronous transfers
extern int wasm_submit_transfer(struct usbi_transfer *itransfer);
extern int wasm_cancel_transfer(struct usbi_transfer *itransfer);
extern void wasm_clear_transfer_priv(struct usbi_transfer *itransfer);

// Event handling & timing
extern int wasm_handle_events(struct libusb_context *ctx, void *event_data, unsigned int count, unsigned int num_ready);
//extern int wasm_handle_transfer_completion(struct usbi_transfer *itransfer); If the function above is implemented, this is not neeeded
// extern int wasm_clock_gettime(int clkid, struct timespec *tp);
// extern clockid_t wasm_get_timerfd_clockid(void);


// Define the os_backend struct that libusb will use.
struct usbi_os_backend wasm_os_backend = {
    .name                      = "wasm",
    .caps                      = LIBUSB_CAP_HAS_HOTPLUG | LIBUSB_CAP_HAS_NO_DETACH_KERNEL_DRIVER, // Example capabilities
    .init                      = wasm_init,
    .exit                      = wasm_exit,
    .set_option                = wasm_set_option,
    .get_device_list           = wasm_get_device_list,
    .hotplug_poll              = wasm_hotplug_poll,
    .open                      = wasm_open_device,
    .close                     = wasm_close_device,
    .destroy_device            = wasm_destroy_device,

    .get_active_config_descriptor = wasm_get_active_config_descriptor,
    .get_config_descriptor     = wasm_get_config_descriptor,
    .get_config_descriptor_by_value = wasm_get_config_descriptor_by_value,

    .get_configuration         = wasm_get_configuration,
    .set_configuration         = wasm_set_configuration,
    .claim_interface           = wasm_claim_interface,
    .release_interface         = wasm_release_interface,
    .set_interface_altsetting  = wasm_set_interface_altsetting,
    .clear_halt                = wasm_clear_halt,
    .reset_device              = wasm_reset_device,

    .alloc_stream              = wasm_alloc_stream,
    .free_stream               = wasm_free_stream,
    .dev_mem_alloc             = wasm_dev_mem_alloc,
    .dev_mem_free              = wasm_dev_mem_free,

    .kernel_driver_active      = wasm_kernel_driver_active,
    .detach_kernel_driver      = wasm_detach_kernel_driver,
    .attach_kernel_driver      = wasm_attach_kernel_driver,

    .submit_transfer           = wasm_submit_transfer,
    .cancel_transfer           = wasm_cancel_transfer,
    .clear_transfer_priv       = wasm_clear_transfer_priv,

    .handle_events             = wasm_handle_events,

    // Backend-private data sizes (set to zero; resource abstraction handles data)
    .context_priv_size         = 0,
    .device_priv_size          = 0,
    .device_handle_priv_size   = 0,
    .transfer_priv_size        = 0
};

// ---------------------------------------------------------------------------
// Initialization & teardown
// ---------------------------------------------------------------------------
int wasm_init(struct libusb_context *ctx)
{
    // (none) – no usb-host call
    return 0;
}

void wasm_exit(struct libusb_context *ctx)
{
    // (none) – no usb-host call
}

int wasm_set_option(struct libusb_context *ctx, enum libusb_option option, va_list value)
{
    // (none) – no usb-host call
    return 0;
}

// ---------------------------------------------------------------------------
// Device discovery
// ---------------------------------------------------------------------------
int wasm_get_device_list(struct libusb_context *ctx, struct discovered_devs **discdevs)
{
    // Calls: usb-host::list-devices() -> list<device>
    // Mapping:
    //    (none) input
    //    (output) C discovered_devs** ← allocate from list<device>
    return 0;
}

void wasm_hotplug_poll(void)
{
    // No direct call here.
    // Hotplug events triggered passively via usb-host::register-hotplug-callback
}

int wasm_wrap_sys_device(struct libusb_device *dev, struct libusb_device_handle *handle, intptr_t sys_dev)
{
    // (none) – Not available in WIT API
    return LIBUSB_ERROR_NOT_SUPPORTED;
}

// ---------------------------------------------------------------------------
// Open / close
// ---------------------------------------------------------------------------
int wasm_open_device(struct libusb_device_handle *handle)
{
    // Calls: usb-host::open(device: device) -> result<handle, errno>
    // Mapping:
    //    input: handle->dev (C pointer) → device (WIT resource)
    //    output: set handle->os_priv to the resulting handle
    return 0;
}

void wasm_close_device(struct libusb_device_handle *handle)
{
    // Automatic resource drop: usb-host::handle dropped
}

void wasm_destroy_device(struct libusb_device *dev)
{
    // Automatic resource drop: usb-host::device dropped
}

// ---------------------------------------------------------------------------
// Descriptor retrieval
// ---------------------------------------------------------------------------
int wasm_get_active_config_descriptor(struct libusb_device *dev,
                                       unsigned char *buf, size_t len, int *host_endian)
{
    // Calls: usb-host::get-config-descriptor-by-value(device: device, value: u8)
    // Mapping:
    //    input: dev (C struct) → device (WIT resource)
    //    input: active config value (retrieved separately)
    //    output: config-descriptor.raw (list<u8>) copied into buf (unsigned char*)
    return 0;
}

int wasm_get_config_descriptor(struct libusb_device *dev, uint8_t index,
                                unsigned char *buf, size_t len, int *host_endian)
{
    // Calls: usb-host::get-config-descriptor-by-index(device: device, index: u8)
    // Mapping:
    //    input: dev (libusb_device*) → device (WIT resource)
    //    input: index (uint8_t) → index (u8)
    //    output: config-descriptor.raw (list<u8>) copied into buf
    return 0;
}

int wasm_get_config_descriptor_by_value(struct libusb_device *dev, uint8_t cfg_value,
                                        unsigned char **buffer, int *host_endian)
{
    // Calls: usb-host::get-config-descriptor-by-value(device: device, value: u8)
    // Mapping:
    //    input: dev (C struct) → device (WIT resource)
    //    input: cfg_value (uint8_t) → value (u8)
    //    output: list<u8> into allocated buffer
    return 0;
}

// ---------------------------------------------------------------------------
// Config & interface management
// ---------------------------------------------------------------------------
int wasm_get_configuration(struct libusb_device_handle *handle, int *cfg)
{
    // Calls: usb-host::get-config(handle: handle) -> u8
    // Mapping:
    //    input: handle (libusb_device_handle*) → handle (WIT resource)
    //    output: u8 config → store into int *cfg
    return 0;
}

int wasm_set_configuration(struct libusb_device_handle *handle, int config)
{
    // Calls: usb-host::set-config(handle: handle, cfg: u8)
    // Mapping:
    //    input: handle (C struct) → handle (WIT resource)
    //    input: config (int) → cfg (u8)
    return 0;
}

int wasm_claim_interface(struct libusb_device_handle *handle, int iface)
{
    // Calls: usb-host::claim-interface(handle: handle, iface: u8)
    // Mapping:
    //    input: iface (int) → u8
    return 0;
}

int wasm_release_interface(struct libusb_device_handle *handle, int iface)
{
    // Calls: usb-host::release-interface(handle: handle, iface: u8)
    return 0;
}

int wasm_set_interface_altsetting(struct libusb_device_handle *handle, int iface, int alt)
{
    // Calls: usb-host::set-altsetting(handle: handle, iface: u8, alt: u8)
    // Mapping:
    //    input: iface (int) → u8
    //    input: alt (int) → u8
    return 0;
}

// ---------------------------------------------------------------------------
// Endpoint control
// ---------------------------------------------------------------------------
int wasm_clear_halt(struct libusb_device_handle *handle, unsigned char endpoint)
{
    // Calls: usb-host::clear-halt(handle: handle, endpoint: u8)
    // Mapping:
    //    input: endpoint (unsigned char) → u8
    return 0;
}

int wasm_reset_device(struct libusb_device_handle *handle)
{
    // Calls: usb-host::reset(handle: handle)
    return 0;
}

// ---------------------------------------------------------------------------
// USB 3 bulk streams
// ---------------------------------------------------------------------------
int wasm_alloc_stream(struct libusb_device_handle *handle, uint32_t num_streams,
                      unsigned char *endpoints, int num_endpoints)
{
    // Calls: usb-host::alloc-streams(handle: handle, num-streams: u32, endpoints: list<u8>)
    // Mapping:
    //    input: num_streams (uint32_t) → u32
    //    input: endpoints[] (unsigned char[]) → list<u8>
    return 0;
}

int wasm_free_stream(struct libusb_device_handle *handle,
                     unsigned char *endpoints, int num_endpoints)
{
    // Calls: usb-host::free-streams(handle: handle, endpoints: list<u8>)
    return 0;
}

// ---------------------------------------------------------------------------
// Memory helpers
// ---------------------------------------------------------------------------
void *wasm_dev_mem_alloc(struct libusb_device *dev, size_t len)
{
    // (none) – malloc(len)
    return malloc(len);
}

void wasm_dev_mem_free(struct libusb_device *dev, void *buffer, size_t len)
{
    // (none) – free(buffer)
    free(buffer);
}

// ---------------------------------------------------------------------------
// Kernel driver management
// ---------------------------------------------------------------------------
int wasm_kernel_driver_active(struct libusb_device_handle *handle, int iface)
{
    // Calls: usb-host::kernel-driver-active(handle: handle, iface: u8)
    return 0;
}

int wasm_detach_kernel_driver(struct libusb_device_handle *handle, int iface)
{
    // Calls: usb-host::detach-kernel-driver(handle: handle, iface: u8)
    return 0;
}

int wasm_attach_kernel_driver(struct libusb_device_handle *handle, int iface)
{
    // Calls: usb-host::attach-kernel-driver(handle: handle, iface: u8)
    return 0;
}

// ---------------------------------------------------------------------------
// Transfer management
// ---------------------------------------------------------------------------
int wasm_submit_transfer(struct usbi_transfer *itransfer)
{
    // Calls:
    //   usb-host::new-transfer(handle: handle, type: xfer-type, setup?: xfer-setup, buf-size: u32, opts: xfer-options)
    //   usb-host::submit-transfer(transfer: transfer, data: list<u8>)
    //   usb-host::await-transfer(transfer: transfer) -> list<u8>
    return 0;
}

int wasm_cancel_transfer(struct usbi_transfer *itransfer)
{
    // Calls: usb-host::cancel-transfer(transfer: transfer)
    return 0;
}

void wasm_clear_transfer_priv(struct usbi_transfer *itransfer)
{
    // (none) – internal state cleanup
}

// ---------------------------------------------------------------------------
// Event handling
// ---------------------------------------------------------------------------
int wasm_handle_events(struct libusb_context *ctx, void *event_data,
                       unsigned int count, unsigned int num_ready)
{
    // (none) – events already handled by awaiting transfers
    return 0;
}