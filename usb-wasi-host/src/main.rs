use crate::component::usb::events::DeviceConnectionEvent;
use crate::component::usb::usb::{
    ConfigDescriptor, DeviceDescriptor, DeviceHandleError, Duration, HostContext, HostDeviceHandle,
    HostUsbDevice, Language, Speed,
};
use std::future::Future;
use wasmtime::component::*;
use wasmtime::{Config, Result};
use wasmtime::{Engine, Store};
use wasmtime_wasi::bindings::Command;
use wasmtime_wasi::{IoView, WasiCtx, WasiCtxBuilder, WasiView};

pub struct UsbDevice {}
pub struct UsbContext {}
pub struct DeviceHandle {}

bindgen!({
    world: "imports",
    path: "../wit",


    with: {
        "component:usb/usb/usb-device": UsbDevice,
        "component:usb/usb/context": UsbContext,
        "component:usb/usb/device-handle": DeviceHandle,
    },
    trappable_imports: true,
    async: true
});

struct MyState {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl MyState {
    pub fn new() -> Self {
        Self {
            table: ResourceTable::new(),
            ctx: WasiCtxBuilder::new().inherit_stdio().build(),
        }
    }
}

impl component::usb::types::Host for MyState {}
impl component::usb::descriptors::Host for MyState {}

impl HostDeviceHandle for MyState {
    async fn device(&mut self, self_: Resource<DeviceHandle>) -> Result<Resource<UsbDevice>> {
        todo!()
    }

    async fn active_configuration(
        &mut self,
        self_: Resource<DeviceHandle>,
    ) -> Result<std::result::Result<u8, ()>> {
        todo!()
    }

    async fn set_active_configuration(
        &mut self,
        self_: Resource<DeviceHandle>,
        config: u8,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn unconfigure(
        &mut self,
        self_: Resource<DeviceHandle>,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn reset(
        &mut self,
        self_: Resource<DeviceHandle>,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn clear_halt(
        &mut self,
        self_: Resource<DeviceHandle>,
        endpoint: u8,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn kernel_driver_active(
        &mut self,
        self_: Resource<DeviceHandle>,
    ) -> Result<std::result::Result<bool, ()>> {
        todo!()
    }

    async fn detach_kernel_driver(
        &mut self,
        self_: Resource<DeviceHandle>,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn attach_kernel_driver(
        &mut self,
        self_: Resource<DeviceHandle>,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn set_auto_attach_detach_kernel_driver(
        &mut self,
        self_: Resource<DeviceHandle>,
        auto: bool,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn claim_interface(
        &mut self,
        self_: Resource<DeviceHandle>,
        iface: u8,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn release_interface(
        &mut self,
        self_: Resource<DeviceHandle>,
        iface: u8,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn set_interface_alt_setting(
        &mut self,
        self_: Resource<DeviceHandle>,
        iface: u8,
        alt_setting: u8,
    ) -> Result<std::result::Result<(), ()>> {
        todo!()
    }

    async fn read_interrupt(
        &mut self,
        self_: Resource<DeviceHandle>,
        endpoint: u8,
        timeout: Duration,
    ) -> Result<std::result::Result<(u64, Vec<u8>), DeviceHandleError>> {
        todo!()
    }

    async fn write_interrupt(
        &mut self,
        self_: Resource<DeviceHandle>,
        endpoint: u8,
        data: Vec<u8>,
        timeout: Duration,
    ) -> Result<std::result::Result<u64, DeviceHandleError>> {
        todo!()
    }

    async fn read_bulk(
        &mut self,
        self_: Resource<DeviceHandle>,
        endpoint: u8,
        max_size: u64,
        timeout: Duration,
    ) -> Result<std::result::Result<(u64, Vec<u8>), DeviceHandleError>> {
        todo!()
    }

    async fn write_bulk(
        &mut self,
        self_: Resource<DeviceHandle>,
        endpoint: u8,
        data: Vec<u8>,
        timeout: Duration,
    ) -> Result<std::result::Result<u64, DeviceHandleError>> {
        todo!()
    }

    async fn read_control(
        &mut self,
        self_: Resource<DeviceHandle>,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        max_size: u16,
        timeout: Duration,
    ) -> Result<std::result::Result<(u64, Vec<u8>), DeviceHandleError>> {
        todo!()
    }

    async fn write_control(
        &mut self,
        self_: Resource<DeviceHandle>,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        data: Vec<u8>,
        timeout: Duration,
    ) -> Result<std::result::Result<u64, DeviceHandleError>> {
        todo!()
    }

    async fn read_languages(
        &mut self,
        self_: Resource<DeviceHandle>,
        timeout: Duration,
    ) -> Result<std::result::Result<Vec<Language>, ()>> {
        todo!()
    }

    async fn read_string_descriptor_ascii(
        &mut self,
        self_: Resource<DeviceHandle>,
        index: u8,
    ) -> Result<std::result::Result<String, ()>> {
        todo!()
    }

    async fn read_string_descriptor(
        &mut self,
        self_: Resource<DeviceHandle>,
        language: Language,
        index: u8,
        timeout: Duration,
    ) -> Result<std::result::Result<String, ()>> {
        todo!()
    }

    async fn read_manufacturer_string_ascii(
        &mut self,
        self_: Resource<DeviceHandle>,
        device: DeviceDescriptor,
    ) -> Result<()> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<DeviceHandle>) -> Result<()> {
        todo!()
    }
}

impl HostUsbDevice for MyState {
    async fn device_descriptor(
        &mut self,
        self_: Resource<UsbDevice>,
    ) -> Result<std::result::Result<DeviceDescriptor, ()>> {
        todo!()
    }

    async fn config_descriptor(
        &mut self,
        self_: Resource<UsbDevice>,
    ) -> Result<std::result::Result<ConfigDescriptor, ()>> {
        todo!()
    }

    async fn bus_number(&mut self, self_: Resource<UsbDevice>) -> Result<u8> {
        todo!()
    }

    async fn address(&mut self, self_: Resource<UsbDevice>) -> Result<u8> {
        todo!()
    }

    async fn speed(&mut self, self_: Resource<UsbDevice>) -> Result<Speed> {
        todo!()
    }

    async fn open(
        &mut self,
        self_: Resource<UsbDevice>,
    ) -> Result<std::result::Result<Resource<DeviceHandle>, ()>> {
        todo!()
    }

    async fn port_number(&mut self, self_: Resource<UsbDevice>) -> Result<u8> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<UsbDevice>) -> Result<()> {
        todo!()
    }
}

impl HostContext for MyState {
    async fn devices(
        &mut self,
        self_: Resource<UsbContext>,
    ) -> Result<std::result::Result<Vec<Resource<UsbDevice>>, ()>> {
        todo!()
    }

    async fn open_device_with_vid_pid(
        &mut self,
        self_: Resource<UsbContext>,
        vendor_id: u16,
        product_id: u16,
    ) -> Result<Option<Resource<DeviceHandle>>> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<UsbContext>) -> Result<()> {
        todo!()
    }
}

impl component::usb::events::Host for MyState {
    async fn update(&mut self) -> Result<StreamReader<u8>> {
        // an eventhandler should be added to the rusb hotplugbuilder,
        // these events should be sent in some way to a buffer, for example mpsc
        // the update function should then return the next event from the buffer
        todo!()
    }
}

impl component::usb::usb::Host for MyState {
    async fn get_context(&mut self) -> Result<std::result::Result<Resource<UsbContext>, ()>> {
        // make a context and return it, if the context already exists, return it
        Result::Ok(Err(()))
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

#[tokio::main]
async fn main() -> Result<()> {
    // Compile the `Component` that is being run for the application.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_component>", args[0]);
        std::process::exit(1);
    }
    let component_path = &args[1];
    let engine = Engine::new(Config::new().async_support(true).wasm_component_model_async(true))?;
    let component = Component::from_file(&engine, component_path)?;
    let mut linker = Linker::new(&engine);
    Imports::add_to_linker(&mut linker, |state: &mut MyState| state)?;
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    let mut store = Store::new(&engine, MyState::new());
    let command = Command::instantiate_async(&mut store, &component, &linker).await?;
    command.wasi_cli_run().call_run(store).await?.unwrap();
    Ok(())
}

/*
    libusb bekijken webusb, taal die programma zelf gebruken zie compileren naar webusb
    gelijkaardige backend voor libusb voor wasi usb
    applicatie compileren met libusb compilenren naar wasi en wasm
 */