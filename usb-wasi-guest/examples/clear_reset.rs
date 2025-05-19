use wit_bindgen::generate;
generate!({ world: "guest", path: "../wit" });

use component::usb::{
    device,
    configuration::ConfigValue,
};

fn main() {
    device::init().expect("init failed");
    let mut devs = device::list_devices().expect("list_devices failed");
    if devs.is_empty() { return; }
    let handle = devs.remove(0).open().expect("open failed");

    // set to configuration 1
    handle
        .set_configuration(ConfigValue::Value(1))
        .expect("set_configuration failed");

    // clear a stall on endpoint 0x81
    handle.clear_halt(0x81).expect("clear_halt failed");

    // reset the device
    handle.reset_device().expect("reset_device failed");

    // unconfigure (cfg = 0)
    handle
        .set_configuration(ConfigValue::Unconfigured)
        .expect("unconfigure failed");

    handle.close();
}
