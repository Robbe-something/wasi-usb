use crate::component::usb::configuration::ConfigValue;
use crate::component::usb::descriptors::ConfigurationDescriptor;
use crate::component::usb::device::{
    HostDeviceHandle, HostUsbDevice, TransferOptions, TransferSetup,
};
use crate::component::usb::errors::LibusbError;
use crate::component::usb::transfers::{HostTransfer, Transfer};
use libusb_sys::{libusb_alloc_streams, libusb_attach_kernel_driver, libusb_cancel_transfer, libusb_claim_interface, libusb_clear_halt, libusb_close, libusb_context, libusb_detach_kernel_driver, libusb_device, libusb_device_handle, libusb_free_streams, libusb_get_configuration, libusb_kernel_driver_active, libusb_release_interface, libusb_reset_device, libusb_set_configuration, libusb_set_interface_alt_setting, libusb_transfer, libusb_unref_device};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasmtime::component::*;
use wasmtime::{Config, Result};
use wasmtime::{Engine, Store};
use wasmtime_wasi::bindings::Command;
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

pub struct UsbTransfer {
    transfer: *mut libusb_transfer,
    completed: Arc<AtomicBool>,
}
pub struct UsbDevice {
    pub device: *mut libusb_device,
}
pub struct UsbDeviceHandle {
    handle: *mut libusb_device_handle,
}

bindgen!({
    world: "host",
    path: "../wit",
    with: {
        "component:usb/transfers/transfer": UsbTransfer,
        "component:usb/device/usb-device": UsbDevice,
        "component:usb/device/device-handle": UsbDeviceHandle,
    },
    trappable_imports: true,
    async: true

});

struct MyState {
    table: ResourceTable,
    ctx: WasiCtx,
    context: Option<*mut libusb_context>,
}

impl MyState {
    pub fn new() -> Self {
        Self {
            table: ResourceTable::new(),
            ctx: WasiCtxBuilder::new().inherit_stdio().build(),
        }
    }
}

impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl LibusbError {
    /// Convert a raw `libusb_error` integer value to a `LibusbError` variant.
    pub fn from_raw(value: i32) -> Self {
        match value {
            -1 => LibusbError::Io,
            -2 => LibusbError::InvalidParam,
            -3 => LibusbError::Access,
            -4 => LibusbError::NoDevice,
            -5 => LibusbError::NotFound,
            -6 => LibusbError::Busy,
            -7 => LibusbError::Timeout,
            -8 => LibusbError::Overflow,
            -9 => LibusbError::Pipe,
            -10 => LibusbError::Interrupted,
            -11 => LibusbError::NoMem,
            -12 => LibusbError::NotSupported,
            -99 => LibusbError::Other,
            _ => LibusbError::Other, // Default to `Other` for unknown error codes
        }
    }
}

impl component::usb::configuration::Host for MyState {}
impl component::usb::descriptors::Host for MyState {}
impl component::usb::errors::Host for MyState {}

impl HostTransfer for MyState {
    async fn submit_transfer(
        &mut self,
        self_: Resource<Transfer>,
        data: Vec<u8>,
    ) -> Result<std::result::Result<(), component::usb::transfers::LibusbError>> {
        todo!()
    }

    async fn cancel_transfer(
        &mut self,
        self_: Resource<UsbTransfer>,
    ) -> Result<std::result::Result<(), LibusbError>> {
        todo!()
    }

    async fn await_transfer(
        &mut self,
        self_: Resource<Transfer>,
    ) -> Result<std::result::Result<Vec<u8>, component::usb::transfers::LibusbError>> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<UsbTransfer>) -> Result<()> {
        if let Ok(transfer) = self.table.get(&rep) {
            unsafe {
                if !transfer.completed.load(Ordering::SeqCst) {
                    let _ = libusb_cancel_transfer(transfer.transfer);
                } else {
                    // If the transfer is already completed, we can safely drop it
                    // without calling `libusb_cancel_transfer`.
                }
            }
        }
        Ok(())
    }
}

impl component::usb::transfers::Host for MyState {}

impl HostUsbDevice for MyState {
    async fn open(
        &mut self,
        self_: Resource<UsbDevice>,
    ) -> Result<std::result::Result<Resource<UsbDeviceHandle>, LibusbError>> {
        todo!()
    }

    async fn get_configuration_descriptor(
        &mut self,
        self_: Resource<UsbDevice>,
        config_index: u8,
    ) -> Result<std::result::Result<ConfigurationDescriptor, LibusbError>> {
        todo!()
    }

    async fn get_configuration_descriptor_by_value(
        &mut self,
        self_: Resource<UsbDevice>,
        config_value: u8,
    ) -> Result<
        std::result::Result<
            component::usb::device::ConfigurationDescriptor,
            component::usb::device::LibusbError,
        >,
    > {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<UsbDevice>) -> Result<()> {
        if let Ok(device) = self.table.get(&rep) {
            unsafe {
                libusb_unref_device(device.device);
            }
        }
        Ok(())
    }
}

impl HostDeviceHandle for MyState {
    async fn get_configuration(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
    ) -> Result<std::result::Result<u8, LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let mut config: i32 = 0;
            let res = libusb_get_configuration(usb_device_handle.handle, &mut config);
            Ok(match res {
                0.. => Ok(config as u8),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn set_configuration(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        config: ConfigValue,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let config_value = match config {
                ConfigValue::Value(value) => value as i32,
                ConfigValue::Unconfigured => 0,
            };
            let res = libusb_set_configuration(usb_device_handle.handle, config_value);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn claim_interface(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        ifac: u8,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_claim_interface(usb_device_handle.handle, ifac as i32);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn release_interface(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        ifac: u8,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_release_interface(usb_device_handle.handle, ifac as i32);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn set_interface_altsetting(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        ifac: u8,
        alt_setting: u8,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_set_interface_alt_setting(
                usb_device_handle.handle,
                ifac as i32,
                alt_setting as i32,
            );
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn clear_halt(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        endpoint: u8,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_clear_halt(usb_device_handle.handle, endpoint);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn reset_device(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_reset_device(usb_device_handle.handle);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn kernel_driver_active(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        ifac: u8,
    ) -> Result<std::result::Result<bool, LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_kernel_driver_active(usb_device_handle.handle, ifac as i32);
            Ok(match res {
                0 => Ok(false),
                1.. => Ok(true),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn detach_kernel_driver(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        ifac: u8,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_detach_kernel_driver(usb_device_handle.handle, ifac as i32);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn attach_kernel_driver(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        ifac: u8,
    ) -> Result<std::result::Result<(), LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        unsafe {
            let res = libusb_attach_kernel_driver(usb_device_handle.handle, ifac as i32);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn close(&mut self, self_: Resource<UsbDeviceHandle>) -> Result<()> {
        if let Ok(handle) = self.table.get(&self_) {
            unsafe {
                libusb_close(handle.handle);
            }
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<UsbDeviceHandle>) -> Result<()> {
        if let Ok(handle) = self.table.get(&rep) {
            unsafe {
                libusb_close(handle.handle);
            }
        }
        Ok(())
    }

    async fn alloc_streams(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        num_streams: u32,
        endpoints: Vec<u8>,
    ) -> Result<std::result::Result<(), component::usb::device::LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        let num_endpoints = endpoints.len() as i32;
        let endpoints_ptr = endpoints.as_ptr() as *mut u8;
        unsafe {
            let res = libusb_alloc_streams(usb_device_handle.handle, num_streams, endpoints_ptr, num_endpoints);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }
    }

    async fn free_streams(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        endpoints: Vec<u8>,
    ) -> Result<std::result::Result<(), component::usb::device::LibusbError>> {
        let usb_device_handle = self.table.get(&self_).expect("Failed to get device handle");
        let num_endpoints = endpoints.len() as i32;
        let endpoints_ptr = endpoints.as_ptr() as *mut u8;
        unsafe {
            let res = libusb_free_streams(usb_device_handle.handle, endpoints_ptr, num_endpoints);
            Ok(match res {
                0.. => Ok(()),
                _ => Err(LibusbError::from_raw(res)),
            })
        }


    }

    async fn new_transfer(
        &mut self,
        self_: Resource<UsbDeviceHandle>,
        xfer_type: component::usb::device::TransferType,
        setup: TransferSetup,
        buf_size: u32,
        opts: TransferOptions,
    ) -> Result<
        std::result::Result<
            Resource<component::usb::device::Transfer>,
            component::usb::device::LibusbError,
        >,
    > {
        todo!()
    }
}

impl component::usb::device::Host for MyState {
    async fn init(
        &mut self,
    ) -> Result<std::result::Result<(), component::usb::device::LibusbError>> {
        todo!()
    }

    async fn list_devices(
        &mut self,
    ) -> Result<std::result::Result<Vec<Resource<UsbDevice>>, component::usb::device::LibusbError>>
    {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Compile the `Component` that is being run for the application.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_component>", args[0]);
        std::process::exit(1);
    }
    let component_path = &args[1];
    let engine = Engine::new(
        Config::new()
            .async_support(true)
            .wasm_component_model_async(true),
    )?;
    let component = Component::from_file(&engine, component_path)?;
    let mut linker = Linker::new(&engine);
    Host_::add_to_linker(&mut linker, |state: &mut MyState| state)?;
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    let mut store = Store::new(&engine, MyState::new());
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    command.wasi_cli_run().call_run(store).await?.unwrap();
    Ok(())
}

/*
   libusb bekijken webusb, taal die programma zelf gebruken zie compileren naar webusb
   gelijkaardige backend voor libusb voor wasi en wasm
   applicatie compileren met libusb compilenren naar wasi en wasm
*/
