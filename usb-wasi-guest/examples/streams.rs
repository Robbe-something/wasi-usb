use wit_bindgen::generate;
generate!({ world: "guest", path: "../wit" });

use component::usb::{ device, configuration::ConfigValue };

fn main() {
    device::init().expect("init failed");
    let mut devs = device::list_devices().expect("list_devices failed");
    if devs.is_empty() { return; }
    let handle = devs.remove(0).open().expect("open failed");

    // switch to cfg 1
    handle
        .set_configuration(ConfigValue::Value(1))
        .expect("set_configuration failed");

    // allocate USB3 streams on endpoint 0x01
    handle
        .alloc_streams(4, vec![0x01])
        .expect("alloc_streams failed");

    // free those streams
    handle
        .free_streams(vec![0x01])
        .expect("free_streams failed");

    handle.close();
}
