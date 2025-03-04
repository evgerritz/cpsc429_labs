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
    buffer: Mutex<[u8; BUFFER_SIZE]>,
    pos: Mutex<usize>
}

impl kernel::Module for RustMymem {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

        let state = Ref::try_new( Device {
            buffer: Mutex::new([0u8; BUFFER_SIZE]),
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
        let buffer = shared.buffer.lock();
        let mut offset_p = shared.pos.lock();

        if data.is_empty() {
            return Ok(0);
        }

        let mut num_bytes: usize = data.len();
        let max_bytes: usize = buffer.len() - *offset_p;
        if max_bytes < num_bytes {
            num_bytes = max_bytes; 
        }

        if num_bytes + *offset_p > BUFFER_SIZE {
            return Err(EINVAL);
        }
        // Write starting from offset
        data.write_slice(&buffer[*offset_p..][..num_bytes])?;

        *offset_p += num_bytes;

        Ok(num_bytes)
    }

    fn write( shared: RefBorrow<'_, Device>, _: &File,
        data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        if data.is_empty() {
            return Ok(0);
        }

        let mut buffer = shared.buffer.lock();
        let mut offset_p = shared.pos.lock();

        let num_bytes: usize = data.len();

        let new_len = num_bytes + *offset_p;
        if new_len > BUFFER_SIZE {
            return Err(EINVAL);
        }

        data.read_slice(&mut buffer[*offset_p..][..num_bytes])?;
        *offset_p += num_bytes;
        Ok(num_bytes)
    }

    fn seek( shared: RefBorrow<'_, Device>, _file: &File,
        offset: SeekFrom) -> Result<u64> {
        let mut old_offset = shared.pos.lock();
        let new_offset: usize;

        match offset {
            SeekFrom::Start(val) => new_offset = val as usize,
            SeekFrom::End(val) => new_offset = BUFFER_SIZE + val as usize,
            SeekFrom::Current(val) => new_offset = *old_offset + val as usize,
        }
        
        if new_offset > BUFFER_SIZE {
            return Err(EINVAL);
        }

        *old_offset = new_offset;
        Ok(new_offset as u64)
    }
}
