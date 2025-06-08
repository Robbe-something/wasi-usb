mod main;

use rusb::{Context, DeviceHandle, GlobalContext, UsbContext};
use std::fs;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::time::{Duration, Instant};
use std::{thread, time};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use mbrman::{MBR, MBRPartitionEntry};
use exfat::{ExFat, directory::Item};

// Custom IoSlice to restrict reads to a partition
struct IoSlice<T: Read + Seek> {
    inner: T,
    start: u64,
    end: u64,
    position: u64,
}

impl<T: Read + Seek> IoSlice<T> {
    fn new(mut inner: T, start: u64, end: u64) -> Result<Self> {
        if start >= end {
            return Err(anyhow!("Invalid slice range"));
        }
        inner.seek(SeekFrom::Start(start))?;
        Ok(IoSlice {
            inner,
            start,
            end,
            position: start,
        })
    }
}

impl<T: Read + Seek> Read for IoSlice<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.end {
            return Ok(0);
        }
        let max_bytes = (self.end - self.position).min(buf.len() as u64) as usize;
        self.inner.seek(SeekFrom::Start(self.position))?;
        let bytes_read = self.inner.read(&mut buf[..max_bytes])?;
        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl<T: Read + Seek> Seek for IoSlice<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => self.start + offset,
            SeekFrom::End(offset) => self.end.saturating_add(offset as u64),
            SeekFrom::Current(offset) => self.position.saturating_add_signed(offset),
        };
        if new_pos < self.start || new_pos > self.end {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek out of range"));
        }
        self.position = new_pos;
        self.inner.seek(SeekFrom::Start(self.position))?;
        Ok(self.position - self.start)
    }
}

// USB mass storage device wrapper using rusb
struct UsbMassStorage<C: UsbContext> {
    handle: DeviceHandle<C>,
    in_ep: u8,
    out_ep: u8,
    block_size: u32,
    seek_position: u64,
    capacity: DeviceCapacity,
    tag: u32,
    interface_number: u8,
}

#[derive(Default, Clone, Copy)]
struct DeviceCapacity {
    size: u64,          // Total size in bytes
    block_length: u32,  // Block size in bytes
}

impl<C: UsbContext> UsbMassStorage<C> {
    fn new(mut handle: DeviceHandle<C>, desc: rusb::DeviceDescriptor, interface_number: u8) -> Result<Self> {

        // Detach kernel driver if necessary
        if handle.kernel_driver_active(interface_number)? {
            handle.detach_kernel_driver(interface_number)?;
        }
        handle.claim_interface(interface_number)?;

        // Find bulk endpoints from the interface descriptor
        let config_desc = handle.device().active_config_descriptor()?;
        let (in_ep, out_ep) = find_bulk_endpoints(&config_desc, interface_number)
            .ok_or_else(|| anyhow!("No bulk endpoints found"))?;

        let mut storage = UsbMassStorage {
            handle,
            in_ep,
            out_ep,
            block_size: 512,
            seek_position: 0,
            capacity: Default::default(),
            tag: 1,
            interface_number,
        };

        storage.reset()?;

        storage.test_unit_ready()?;

        storage.capacity = storage.read_capacity()?;

        Ok(storage)
    }

    // Issue a Bulk-Only Mass Storage Reset via a class-specific control transfer
    fn reset(&mut self) -> Result<()> {
        let request_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Class,
            rusb::Recipient::Interface,
        );
        let request = 0xFF;
        let value = 0;
        let index = self.interface_number as u16;
        self.handle
            .write_control(request_type, request, value, index, &[], Duration::from_secs(1))?;
        // Short delay to let device process reset
        thread::sleep(time::Duration::from_millis(100));
        Ok(())
    }

    // SCSI TEST UNIT READY (no data phase)
    fn test_unit_ready(&mut self) -> Result<()> {
        let cbwcb = vec![0x00, 0, 0, 0, 0, 0]; // TEST UNIT READY
        self.send_cbw(&cbwcb, None)?;
        Ok(())
    }

    // SCSI READ CAPACITY (10) -> returns DeviceCapacity
    fn read_capacity(&mut self) -> Result<DeviceCapacity> {
        let mut data = [0u8; 8];
        let mut cbwcb = BytesMut::with_capacity(10);
        cbwcb.put_u8(0x25); // READ CAPACITY (10)
        cbwcb.put_bytes(0, 9);

        self.receive_cbw(&cbwcb.to_vec(), &mut data)?;
        let mut buf = Bytes::from(data.to_vec());
        let last_lba = buf.get_u32();
        let block_len = buf.get_u32();
        // The device reports last LBA, so total blocks = last_lba + 1
        let total_blocks = (last_lba as u64) + 1;
        let size = total_blocks * (block_len as u64);

        Ok(DeviceCapacity {
            size,
            block_length: block_len,
        })
    }

    // Helper: send CBW and no data, then read CSW
    fn send_cbw(&mut self, cbwcb: &[u8], data_out: Option<&[u8]>) -> Result<()> {
        let tag = self.next_tag();
        let data_len = data_out.map(|d| d.len() as u32).unwrap_or(0);

        // Construct CBW
        let mut cbw = BytesMut::with_capacity(31);
        cbw.put_u32_le(0x4342_5355);           // dCBWSignature 'USBC'
        cbw.put_u32_le(tag);                  // dCBWTag
        cbw.put_u32_le(data_len);             // dCBWDataTransferLength
        cbw.put_u8(if data_out.is_some() { 0x00 } else { 0x00 }); // bmCBWFlags: OUT=0x00, IN=0x80 (none here)
        cbw.put_u8(0);                        // bCBWLUN = 0
        cbw.put_u8(cbwcb.len() as u8);        // bCBWCBLength
        cbw.put_slice(cbwcb);                 // CBWCB
        cbw.resize(31, 0);

        // Send CBW to OUT endpoint
        let written = self
            .handle
            .write_bulk(self.out_ep, &cbw, Duration::from_secs(1))?;
        if written != 31 {
            return Err(anyhow!("CBW write short: {} bytes", written));
        }

        // If there's data to send (DATA OUT), send it
        if let Some(d) = data_out {
            let mut offset = 0;
            while offset < d.len() {
                let chunk = &d[offset..];
                let written = self
                    .handle
                    .write_bulk(self.out_ep, chunk, Duration::from_secs(1))?;
                offset += written;
            }
        }

        // Finally, read CSW
        self.read_csw(tag)?;
        Ok(())
    }

    // Helper: send CBW expecting data IN, read data, then read CSW
    fn receive_cbw(&mut self, cbwcb: &[u8], data_in: &mut [u8]) -> Result<()> {
        let tag = self.next_tag();
        let data_len = data_in.len() as u32;

        // Construct CBW
        let mut cbw = BytesMut::with_capacity(31);
        cbw.put_u32_le(0x4342_5355);    // dCBWSignature 'USBC'
        cbw.put_u32_le(tag);           // dCBWTag
        cbw.put_u32_le(data_len);      // dCBWDataTransferLength
        cbw.put_u8(0x80);              // bmCBWFlags: IN = 0x80
        cbw.put_u8(0);                 // bCBWLUN = 0
        cbw.put_u8(cbwcb.len() as u8);// bCBWCBLength
        cbw.put_slice(cbwcb);          // CBWCB
        cbw.resize(31, 0);

        // Send CBW
        let written = self
            .handle
            .write_bulk(self.out_ep, &cbw, Duration::from_secs(1))?;
        if written != 31 {
            return Err(anyhow!("CBW write short: {} bytes", written));
        }

        // Read data phase
        let mut total_read = 0;
        while total_read < data_len as usize {
            let chunk = &mut data_in[total_read..];
            let len = self
                .handle
                .read_bulk(self.in_ep, chunk, Duration::from_secs(1))?;
            if len == 0 {
                break;
            }
            total_read += len;
        }

        // Read CSW
        self.read_csw(tag)?;
        Ok(())
    }

    // Read the 13-byte CSW and validate
    fn read_csw(&mut self, expected_tag: u32) -> Result<()> {
        let mut csw = [0u8; 13];
        let mut total_read = 0;
        while total_read < 13 {
            let len = self
                .handle
                .read_bulk(self.in_ep, &mut csw[total_read..], Duration::from_secs(1))?;
            if len == 0 {
                break;
            }
            total_read += len;
        }
        if total_read < 13 {
            return Err(anyhow!("CSW too short: {} bytes", total_read));
        }

        let sig = u32::from_le_bytes([csw[0], csw[1], csw[2], csw[3]]);
        if sig != 0x5342_5355 {
            return Err(anyhow!("Invalid CSW signature: 0x{:08x}", sig));
        }
        let tag = u32::from_le_bytes([csw[4], csw[5], csw[6], csw[7]]);
        if tag != expected_tag {
            return Err(anyhow!(
                "CSW tag mismatch: got {}, expected {}",
                tag,
                expected_tag
            ));
        }
        let status = csw[12];
        if status != 0 {
            return Err(anyhow!("CSW indicates failure, status {}", status));
        }
        Ok(())
    }

    // SCSI READ (10): block_address = LBA, transfer_length = number of blocks
    fn read_10(&mut self, block_address: u32, transfer_length: u16) -> Result<Vec<u8>> {
        let mut cbwcb = BytesMut::with_capacity(10);
        cbwcb.put_u8(0x28); // READ(10)
        cbwcb.put_u8(0);
        cbwcb.put_slice(&block_address.to_be_bytes());
        cbwcb.put_u8(0);
        cbwcb.put_slice(&transfer_length.to_be_bytes());
        cbwcb.put_u8(0);

        let data_len = (transfer_length as u64 * self.capacity.block_length as u64) as usize;
        let mut buffer = vec![0u8; data_len];
        let start = Instant::now();
        self.receive_cbw(&cbwcb.to_vec(), &mut buffer)?;
        let elapsed = start.elapsed();
        let kb_s = if elapsed.as_millis() > 0 {
            (data_len as f64 / 1024.0) / (elapsed.as_millis() as f64 / 1000.0)
        } else {
            0.0
        };
        Ok(buffer)
    }

    fn next_tag(&mut self) -> u32 {
        let t = self.tag;
        self.tag = self.tag.wrapping_add(1);
        t
    }
}

impl<C: UsbContext> Read for UsbMassStorage<C> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let start_addr = self.seek_position;
        let end_addr = self.seek_position + buf.len() as u64;
        if end_addr > self.capacity.size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Read exceeds capacity",
            ));
        }
        let block_len = self.capacity.block_length as u64;
        let block_addr = (start_addr / block_len) as u32;
        let blocks_needed = ((end_addr - 1) / block_len - block_addr as u64 + 1) as u16;

        let data = self
            .read_10(block_addr, blocks_needed)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("USB error: {:?}", e)))?;

        let offset_in_block = (start_addr % block_len) as usize;
        let bytes_available = (blocks_needed as u64 * block_len) as usize;
        let copy_len = buf.len().min(bytes_available - offset_in_block);
        buf.copy_from_slice(&data[offset_in_block..offset_in_block + copy_len]);

        self.seek_position += copy_len as u64;
        Ok(copy_len)
    }
}

impl<C: UsbContext> Seek for UsbMassStorage<C> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.seek_position = match pos {
            SeekFrom::Start(off) => off,
            SeekFrom::End(off) => {
                if off >= 0 {
                    self.capacity.size.saturating_add(off as u64)
                } else {
                    self.capacity.size.saturating_sub((-off) as u64)
                }
            }
            SeekFrom::Current(off) => {
                if off >= 0 {
                    self.seek_position.saturating_add(off as u64)
                } else {
                    self.seek_position.saturating_sub((-off) as u64)
                }
            }
        };
        Ok(self.seek_position)
    }
}

// Find bulk‐IN and bulk‐OUT endpoints for a given interface
fn find_bulk_endpoints(
    config: &rusb::ConfigDescriptor,
    interface_number: u8,
) -> Option<(u8, u8)> {
    for interface in config.interfaces() {
        if interface.number() != interface_number {
            continue;
        }
        for descriptor in interface.descriptors() {
            let mut in_ep = None;
            let mut out_ep = None;
            for endpoint in descriptor.endpoint_descriptors() {
                let addr = endpoint.address();
                match endpoint.transfer_type() {
                    rusb::TransferType::Bulk => {
                        if addr & rusb::constants::LIBUSB_ENDPOINT_DIR_MASK == rusb::constants::LIBUSB_ENDPOINT_IN {
                            in_ep = Some(addr);
                        } else {
                            out_ep = Some(addr);
                        }
                    }
                    _ => {}
                }
            }
            if let (Some(i), Some(o)) = (in_ep, out_ep) {
                return Some((i, o));
            }
        }
    }
    None
}

fn read_item<T: Seek + Read>(item: Item<T>) -> Result<u64> {
    match item {
        Item::File(mut file) => {
            let size = file.len();
            if let Some(mut handle) = file.open()? {
                let mut buffer = vec![0u8; size as usize];
                handle.read_exact(&mut buffer)?;
                Ok(size)
            } else {
                Ok(0)
            }
        }
        Item::Directory(dir) => {
            let mut total = 0;
            for child in dir.open()? {
                total += read_item(child)?;
            }
            Ok(total)
        }
    }
}

fn main() -> Result<()> {
    thread::sleep(time::Duration::from_secs(10));
    // Initialize libusb context
    let context = Context::new().context("Failed to initialize rusb context")?;

    // Find target device (0951:1666)
    let mut found = None;
    for device in context.devices()?.iter() {
        let desc = device.device_descriptor()?;
        if desc.vendor_id() == 0x0951 && desc.product_id() == 0x1666 {
            found = Some((device, desc));
            break;
        }
    }
    let (device, desc) = match found {
        Some(pair) => pair,
        None => {
            eprintln!("USB drive (0951:1666) not found");
            return Ok(());
        }
    };

    // Open device handle
    let mut handle = device.open().context("Failed to open USB device")?;

    // Use interface 0 for mass storage
    let interface_number = 0u8;
    handle
        .set_active_configuration(1)
        .ok(); // Some devices may already be configured

    let mut usb = UsbMassStorage::new(handle, desc, interface_number)
        .context("Failed to initialize mass storage wrapper")?;

    // Read MBR from sector 0
    let block_len = usb.capacity.block_length;
    let mut mbr_buf = vec![0u8; block_len as usize];
    usb.seek(SeekFrom::Start(0))?;
    usb.read_exact(&mut mbr_buf)?;
    let mbr = MBR::read_from(&mut usb, block_len).context("Failed to parse MBR")?;

    let data_partition = match mbr.iter().find(|p| p.1.is_used()) {
        Some((_, p)) => p,
        None => {
            println!("No used partition found");
            return Ok(());
        }
    };

    let slice_start = data_partition.starting_lba as u64 * block_len as u64;
    let slice_end = (data_partition.starting_lba as u64 + data_partition.sectors as u64) * block_len as u64;
    let mut timings = Vec::with_capacity(30);


    for _ in 0..30 {
        // Every iteration: build a fresh IoSlice and BufReader
        let slice = match IoSlice::new(&mut usb, slice_start, slice_end) {
            Ok(slice) => slice,
            Err(e) => {
                println!("Failed to create slice: {:?}", e);
                return Ok(());
            }
        };
        let mut buffered_stream = std::io::BufReader::new(slice);

        let start = Instant::now();
        let image = match ExFat::open(buffered_stream) {
            Ok(image) => image,
            Err(e) => {
                println!("Failed to open exFAT: {:?}", e);
                return Ok(());
            }
        };

        let mut total_bytes = 0;
        for item in image {
            match read_item(item) {
                Ok(bytes) => total_bytes += bytes,
                Err(e) => println!("Error reading item: {:?}", e),
            }
        }

        let duration = start.elapsed().as_millis() as u64;

        timings.push((total_bytes, duration));
    }

    // Write durations to a file for analysis
    let mut file = fs::File::create("throughput_native.txt").expect("Failed to create file");
    for duration in timings {
        writeln!(file, "{}, {}", duration.0, duration.1).expect("Failed to write to file");
    }

    Ok(())
}