use std::fs::File;
use std::io::Write;
use rusb::{Context, Direction, TransferType, UsbContext};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // USB device vendor and product ID
    let vid = 0x046d;
    let pid = 0xc52b;
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
    handle.set_active_configuration(1)?;

    // Retrieve the active configuration descriptor to find an interrupt IN endpoint
    let config = handle.device().active_config_descriptor()?;
    let (iface_num, endpoint_address) = {
        let mut iface_num = 1;
        let mut endpoint_address = 0x82;
        (iface_num, endpoint_address)
    };
    println!("Using interface {} endpoint 0x{:02x}", iface_num, endpoint_address);

    // Detach kernel driver if necessary and claim interface
    if handle.kernel_driver_active(iface_num)? {
        handle.detach_kernel_driver(iface_num)?;
    }
    handle.claim_interface(iface_num)?;

    // Buffer for incoming data
    let mut buf = [0u8; 8];
    // Track time between polls
    let mut last = Instant::now();

    println!("Starting interrupt polling loop (press Ctrl+C to exit)...");
    let mut durations = Vec::with_capacity(50_000);

    for _ in 0..1000 {
        handle.read_interrupt(endpoint_address, &mut buf, timeout)?;
    }

    // Measure timer overhead (Instant::now() + elapsed())
    let timer_overhead_ns: f64 = {
        let t0 = Instant::now();
        for _ in 0..50_000 {
            let _ = Instant::now().elapsed();
        }
        t0.elapsed().as_nanos() as f64 / 50_000 as f64
    };

    durations.clear();
    for i in 0..50_000 {
        // Seek and read without including timer or seek overhead
        let start = Instant::now();
        handle.read_interrupt(endpoint_address, &mut buf, timeout)?;
        let raw_ns = start.elapsed().as_nanos() as f64;
        // Subtract timer overhead to isolate block-read latency
        let read_only_ns = raw_ns - timer_overhead_ns;
        durations.push(read_only_ns as u64);
    }

    let mut file = File::create("latencies_native_interrupt.txt").expect("Failed to create file");
    for duration in durations {
        writeln!(file, "{}", duration).expect("Write failed");
    }

    // (unreachable, but for completeness)
    // handle.release_interface(iface_num)?;
    // println!("Released interface {}", iface_num);
    Ok(())
}