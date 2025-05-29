use rusb::{Context, Direction, TransferType, UsbContext};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // USB device vendor and product ID
    let vid = 0x05e3;
    let pid = 0x0608;
    // Timeout for interrupt transfers
    let timeout = Duration::from_millis(500);

    // Initialize libusb context
    let context = Context::new()?;

    // Open the device by VID/PID
    let mut handle = match context.open_device_with_vid_pid(vid, pid) {
        Some(h) => h,
        None => {
            eprintln!("Device {:04x}:{:04x} not found", vid, pid);
            return Ok(());
        }
    };
    println!("Opened device {:04x}:{:04x}", vid, pid);

    // Retrieve the active configuration descriptor to find an interrupt IN endpoint
    let config = handle.device().active_config_descriptor()?;
    let (iface_num, endpoint_address) = {
        let mut iface_num = 0;
        let mut endpoint_address = 0;
        for iface in config.interfaces() {
            for desc in iface.descriptors() {
                for ep in desc.endpoint_descriptors() {
                    if ep.transfer_type() == TransferType::Interrupt && ep.direction() == Direction::In {
                        iface_num = desc.interface_number();
                        endpoint_address = ep.address();
                        break;
                    }
                }
                if endpoint_address != 0 {
                    break;
                }
            }
            if endpoint_address != 0 {
                break;
            }
        }
        if endpoint_address == 0 {
            return Err(Box::<dyn std::error::Error>::from("No interrupt IN endpoint found"));
        }
        (iface_num, endpoint_address)
    };
    println!("Using interface {} endpoint 0x{:02x}", iface_num, endpoint_address);

    // Detach kernel driver if necessary and claim interface
    if handle.kernel_driver_active(iface_num)? {
        handle.detach_kernel_driver(iface_num)?;
    }
    handle.claim_interface(iface_num)?;

    // Buffer for incoming data
    let mut buf = [0u8; 64];
    // Track time between polls
    let mut last = Instant::now();

    println!("Starting interrupt polling loop (press Ctrl+C to exit)...");
    let mut iteration = 0;
    loop {
        // Compute time since last poll
        let now = Instant::now();
        let elapsed = now.duration_since(last);
        println!("Iteration {}: time since last poll: {:?}", iteration, elapsed);
        last = now;

        // Perform the interrupt transfer
        match handle.read_interrupt(endpoint_address, &mut buf, timeout) {
            Ok(len) => println!("Iteration {}: Read {} bytes: {:?}", iteration, len, &buf[..len]),
            Err(e) => eprintln!("Iteration {}: Error: {:?}", iteration, e),
        }

        iteration += 1;
    }

    // (unreachable, but for completeness)
    // handle.release_interface(iface_num)?;
    // println!("Released interface {}", iface_num);
    // Ok(())
}