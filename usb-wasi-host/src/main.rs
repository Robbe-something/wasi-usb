use std::future::Future;
use wasmtime::component::*;
use wasmtime::Result;
use wasmtime::{Engine, Store};
use component::usb::usb::{HostContext, HostUsbDevice, HostDeviceHandle};

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

#[derive(Default)]
struct MyState {
    table: ResourceTable,
}

impl component::usb::usb::Host for MyState {}

impl HostContext for MyState {

    fn devices(&mut self, self_: Resource<UsbContext>) -> Result<Result<Vec<Resource<UsbDevice>>>> {
        todo!()
    }

    async fn open_device_with_vid_pid(&mut self, self_: Resource<UsbContext>, vendor_id: u16, product_id: u16) -> Result<Option<Resource<DeviceHandle>>> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<UsbContext>) -> Result<()> {
        todo!()
    }
}

impl HostUsbDevice for MyState {}
impl HostDeviceHandle for MyState {}


fn main() -> wasmtime::Result<()> {
    // Compile the `Component` that is being run for the application.
    let engine = Engine::default();
    let component = Component::from_file(&engine, "target/wasm32-wasi/debug/usb.wasm")?;
    let mut linker = Linker::new(&engine);
    Imports::add_to_linker(&mut linker, |state: &mut MyState| state)?;
    let mut store = Store::new(&engine, MyState::default());
    Ok(())
}