// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use kernel::prelude::*;
use kernel::{
    file::{self, File, SeekFrom},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{smutex::Mutex, Ref, RefBorrow},
};

module! {
    type: RustMymem,
    name: "rust_mymem",
    author: "Evan Gerritz",
    description: "mymem test module in Rust",
    license: "GPL",
}

struct RustMymem {
    _dev: Pin<Box<miscdev::Registration<RustMymem>>>,
}

const BUFFER_SIZE: usize = 512*1024;

struct Device {
    buffer: Mutex<Vec<u8>>,
    pos: Mutex<usize>
}

impl kernel::Module for RustMymem {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

        let state = Ref::try_new( Device {
            buffer: Mutex::new(Vec::new()),
            pos: Mutex::new(0)
        })?;

        Ok(RustMymem {                  // 438 == 0o666
            _dev: miscdev::Options::new().mode(438).register_new(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMymem {
    fn drop(&mut self) {
        pr_info!("rust_mymem (exit)\n");
    }
}

#[vtable]
impl file::Operations for RustMymem {
    type OpenData = Ref<Device>;
    type Data = Ref<Device>;

    fn open(shared: &Ref<Device>, _file: &File) -> Result<Self::Data> {
        pr_info!("rust_mymem (open)\n");
        Ok(shared.clone())
    }

    fn read( shared: RefBorrow<'_, Device>, _file: &File,
        data: &mut impl IoBufferWriter, offset: u64 ) -> Result<usize> {
        pr_info!("offset, read: {:?}", offset);
        let buffer = shared.buffer.lock();
        let offset = shared.pos.lock();

        if data.is_empty() {
            return Ok(0);
        }

        let mut num_bytes: usize = data.len();
        let max_bytes: usize = buffer.len() - offset;
        if max_bytes < num_bytes {
            num_bytes = max_bytes; 
        }

        if num_bytes + offset > BUFFER_SIZE {
            return Err(EINVAL);
        }
        // Write starting from offset
        data.write_slice(&buffer[offset..][..num_bytes])?;

        offset += num_bytes;

        Ok(num_bytes)
    }

    fn write( shared: RefBorrow<'_, Device>, _: &File,
        data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        pr_info!("offset, write: {:?}", offset);
        if data.is_empty() {
            return Ok(0);
        }
        let mut buffer = shared.buffer.lock();
        let offset = shared.pos.lock();

        let num_bytes: usize = data.len();

        let new_len = num_bytes + offset;
        if new_len > BUFFER_SIZE {
            return Err(EINVAL);
        }

        if new_len > buffer.len() {
            buffer.try_resize(new_len, 0)?;
        }
        
        data.read_slice(&mut buffer[offset..][..num_bytes])?;
        offset += num_bytes;
        Ok(num_bytes)
    }

    fn seek( shared: RefBorrow<'_, Device>, _file: &File,
        offset: SeekFrom) -> Result<u64> {
        let old_offset = shared.pos.lock();
        let mut new_offset;
        match offset {
            SeekFrom::Start(val) => new_offset = val,
            SeekFrom::End(val) => new_offset = BUFFER_SIZE + val,
            SeekFrom::Current(val) => new_offset = old_offset + val,
        }
        *old_offset = new_offset;
        Ok(new_offset)
    }
}
