use wit_bindgen::generate;
generate!({
    world: "guest",
    path: "../wit",
});

use component::usb::{
    device,
    transfers::{TransferType, TransferSetup, TransferOptions},
};

/// Issue a Control-IN transfer and return the received data.
///
/// Always uses bmRequestType = 0x80 (IN | standard | device).
fn control_in(
    handle: &device::DeviceHandle,
    request: u8,
    w_value: u16,
    w_index: u16,
    len: u16,
) -> Result<Vec<u8>, component::usb::errors::LibusbError> {
    let setup = TransferSetup {
        bm_request_type: 0x80,
        b_request: request,
        w_value,
        w_index,
    };
    let opts = TransferOptions {
        endpoint: 0,          // EP-0
        timeout_ms: 1_000,
        stream_id: 0,
        iso_packets: 0,
    };
    let xfer = handle
        .new_transfer(TransferType::Control, setup, len as u32, opts)
        .expect("new_transfer failed");

    // OUT buffer is empty for IN requests
    xfer.submit_transfer(&[]).expect("submit failed");
    xfer.await_transfer()
}

/// Decode UTF-16LE bytes from a string-descriptor into Rust UTF-8.
fn decode_usb_string(buf: &[u8]) -> String {
    if buf.len() < 2 || buf[1] != 0x03 {
        return "<bad string>".into();
    }
    // bytes 2.. are UTF-16LE code units
    let utf16: Vec<u16> = buf[2..]
        .chunks(2)
        .filter_map(|b| if b.len() == 2 {
            Some(u16::from_le_bytes([b[0], b[1]]))
        } else { None })
        .collect();
    String::from_utf16(&utf16).unwrap_or_else(|_| "<utf16 error>".into())
}

fn main() {
    //----------------------------------------
    // 1: initialise backend
    //----------------------------------------
    device::init().expect("libusb init failed");

    //----------------------------------------
    // 2: list and iterate devices
    //----------------------------------------
    let devs = device::list_devices().expect("list_devices failed");
    if devs.is_empty() {
        println!("No USB devices found.");
        return;
    }
    println!("Found {} device(s)\n", devs.len());

    for (idx, dev) in devs.iter().enumerate() {
        println!("── Device #{} ──", idx);

        //------------------------------------
        // open – skip if permission denied
        //------------------------------------
        let handle = match dev.open() {
            Ok(h) => h,
            Err(e) => {
                println!("  <cannot open>  {:?}", e);
                continue;
            }
        };

        //------------------------------------
        // fetch the 18-byte device descriptor
        //------------------------------------
        let ddesc = match control_in(&handle, 0x06, 0x0100, 0, 18) {
            Ok(buf) => buf,
            Err(e) => {
                println!("  GET_DESCRIPTOR(Device) failed: {:?}", e);
                handle.close();
                continue;
            }
        };
        if ddesc.len() != 18 {
            println!("  Bad descriptor length {}", ddesc.len());
            handle.close();
            continue;
        }
        let vid = u16::from_le_bytes([ddesc[8],  ddesc[9]]);
        let pid = u16::from_le_bytes([ddesc[10], ddesc[11]]);
        let mfg_idx  = ddesc[14];
        let prod_idx = ddesc[15];
        let num_cfg  = ddesc[17];

        println!("  VID:PID            {:04x}:{:04x}", vid, pid);
        println!("  Configurations     {}", num_cfg);

        //------------------------------------
        // fetch manufacturer / product strings (if any)
        //------------------------------------
        let mut mfg = None;
        if mfg_idx != 0 {
            if let Ok(buf) = control_in(&handle, 0x06, 0x0300 | mfg_idx as u16, 0x0409, 255) {
                mfg = Some(decode_usb_string(&buf));
            }
        }
        let mut prod = None;
        if prod_idx != 0 {
            if let Ok(buf) = control_in(&handle, 0x06, 0x0300 | prod_idx as u16, 0x0409, 255) {
                prod = Some(decode_usb_string(&buf));
            }
        }
        if mfg.is_some() || prod.is_some() {
            println!(
                "  Strings            {} {}",
                mfg.as_deref().unwrap_or(""),
                prod.as_deref().unwrap_or("")
            );
        }

        //------------------------------------
        // Optionally list each configuration’s total length
        //------------------------------------
        for cfg in 0..num_cfg {
            if let Ok(buf) = control_in(&handle, 0x06, 0x0200 | cfg as u16, 0, 9) {
                if buf.len() == 9 {
                    let total_len = u16::from_le_bytes([buf[2], buf[3]]);
                    let num_ifaces = buf[4];
                    println!(
                        "  [cfg {:02}] total_len={}  interfaces={}",
                        cfg, total_len, num_ifaces
                    );
                }
            }
        }

        //------------------------------------
        // close handle
        //------------------------------------
        handle.close();
        println!();
    }
}