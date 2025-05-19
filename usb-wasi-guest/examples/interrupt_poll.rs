use wit_bindgen::generate;
generate!({
    world: "guest",
    path: "../wit",
});

use component::usb::{
    device,
    transfers::{TransferType, TransferSetup, TransferOptions},
};
use crate::component::usb::configuration::ConfigValue;
use crate::component::usb::transfers;

fn main() {
    device::init().expect("init failed");
    let mut devs = device::list_devices().expect("list_devices failed");
    if devs.is_empty() {
        println!("No devices.");
        return;
    }
    let handle = devs.remove(0).open().expect("open failed");

    // assume cfg=1, iface=1, int IN @0x81
    handle.set_configuration(ConfigValue::Value(1)).expect("set_configuration");

    // detach kernel driver if active
    if let Ok(true) = handle.kernel_driver_active(1) {
        let _ = handle.detach_kernel_driver(1);
    }

    let res = handle.claim_interface(1);
    if res.is_err() {
        println!("claim_interface failed: {:?}", res);
        return;
    }
    println!("Claimed interface 1");
    handle.set_interface_altsetting(1, 0).expect("altsetting");

    let setup = TransferSetup { bm_request_type: 0, b_request: 0, w_value: 0, w_index: 0 };
    let opts = TransferOptions { endpoint: 0x81, timeout_ms: 2000, stream_id: 0, iso_packets: 0 };

    for i in 0..10 {
        let xfer = handle
            .new_transfer(TransferType::Interrupt, setup, 8, opts)
            .expect("new_transfer");
        xfer.submit_transfer(&[]).expect("submit_transfer");

        match transfers::await_transfer(xfer) {
            Ok(buf) => println!("Iteration {}: {:?} bytes: {:?}", i, buf.len(), buf),
            Err(e)  => println!("Iteration {}: error {:?}", i, e),
        }
    }

    handle.release_interface(1).expect("release_interface");
    handle.close();
}
