

use rusb::{DeviceHandle, GlobalContext, Direction, TransferType};
use std::time::Instant;
use std::fs::File;
use std::io::Write;

const VENDOR_ID: u16 = 0x0951;
const PRODUCT_ID: u16 = 0x1666;
const BLOCK_SIZE: usize = 512;
const INTERFACE: u8 = 0;
const ENDPOINT_IN: u8 = 0x81;
const ENDPOINT_OUT: u8 = 0x02;

fn find_device() -> rusb::Result<DeviceHandle<GlobalContext>> {
    for dev in rusb::devices()?.iter() {
        println!("Found device: {:?}", dev);
        let desc = dev.device_descriptor()?;
        if desc.vendor_id() == VENDOR_ID && desc.product_id() == PRODUCT_ID {
            println!("Matching device found: VID={:04x}, PID={:04x}", VENDOR_ID, PRODUCT_ID);
            let mut handle = dev.open()?;
            println!("Opened device handle");
            if handle.kernel_driver_active(INTERFACE)? {
                handle.detach_kernel_driver(INTERFACE)?;
                println!("Detached kernel driver from interface {}", INTERFACE);
            }
            handle.claim_interface(INTERFACE)?;
            println!("Claimed interface {}", INTERFACE);
            return Ok(handle);
        }
    }
    Err(rusb::Error::NoDevice)
}

fn send_scsi_command(
    handle: &mut DeviceHandle<GlobalContext>,
    cbwcb: &[u8],
    data: Option<&mut [u8]>,
    direction_in: bool,
) -> rusb::Result<()> {
    // Simplified CBW header (not all fields are respected by all devices)
    let tag = 0xdeadbeef_u32;
    let data_len = data.as_ref().map_or(0, |d| d.len()) as u32;

    let mut cbw = vec![0u8; 31];
    cbw[0..4].copy_from_slice(b"USBC"); // CBW Signature
    cbw[4..8].copy_from_slice(&tag.to_le_bytes());
    cbw[8..12].copy_from_slice(&data_len.to_le_bytes());
    cbw[12] = if direction_in { 0x80 } else { 0x00 }; // Flags
    cbw[13] = 0; // LUN
    cbw[14] = cbwcb.len() as u8;
    cbw[15..15 + cbwcb.len()].copy_from_slice(cbwcb);

    handle.write_bulk(ENDPOINT_OUT, &cbw, std::time::Duration::from_secs(1))?;

    if let Some(buffer) = data {
        if direction_in {
            handle.read_bulk(ENDPOINT_IN, buffer, std::time::Duration::from_secs(1))?;
        } else {
            handle.write_bulk(ENDPOINT_OUT, buffer, std::time::Duration::from_secs(1))?;
        }
    }

    let mut csw = [0u8; 13];
    handle.read_bulk(ENDPOINT_IN, &mut csw, std::time::Duration::from_secs(1))?;
    Ok(())
}

fn read_block(handle: &mut DeviceHandle<GlobalContext>, lba: u32, buffer: &mut [u8]) -> rusb::Result<()> {
    let mut cbwcb = [0u8; 10];
    cbwcb[0] = 0x28; // READ(10)
    cbwcb[2..6].copy_from_slice(&lba.to_be_bytes());
    cbwcb[7..9].copy_from_slice(&(1u16.to_be_bytes())); // transfer 1 block
    send_scsi_command(handle, &cbwcb, Some(buffer), true)
}

fn main() -> rusb::Result<()> {
    let mut handle = find_device()?;
    println!("Device connected");

    const WARMUP_ITERS: usize  = 1_000;
    const MEASURE_ITERS: usize = 1_000_000;
    let mut durations = Vec::with_capacity(MEASURE_ITERS);
    let mut buffer = vec![0u8; BLOCK_SIZE];
    // Warm up caches and device
    for _ in 0..WARMUP_ITERS {
        read_block(&mut handle, 0, &mut buffer)?;
    }
    // Measure timer overhead (Instant::now() + elapsed())
    let timer_overhead_ns: f64 = {
        let t0 = Instant::now();
        for _ in 0..MEASURE_ITERS {
            let _ = Instant::now().elapsed();
        }
        t0.elapsed().as_nanos() as f64 / MEASURE_ITERS as f64
    };
    durations.clear();
    for i in 0..MEASURE_ITERS {
        // Seek and read without including timer or seek overhead
        let start = Instant::now();
        read_block(&mut handle, i as u32, &mut buffer)?;
        let raw_ns = start.elapsed().as_nanos() as f64;
        // Subtract timer overhead to isolate block-read latency
        let read_only_ns = raw_ns - timer_overhead_ns;
        durations.push(read_only_ns as u64);
    }

    let mut file = File::create("latencies_native.txt").expect("Failed to create file");
    for duration in durations {
        writeln!(file, "{}", duration).expect("Write failed");
    }

    Ok(())
}