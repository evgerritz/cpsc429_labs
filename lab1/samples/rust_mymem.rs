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
}

impl kernel::Module for RustMymem {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

        let state = Ref::try_new( Device {
            buffer: Mutex::new(Vec::new())
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

    fn open(shared: &Ref<Device>, file: &File) -> Result<Self::Data> {
        pr_info!("rust_mymem (open)\n");
        if file.flags() & file::flags::O_ACCMODE == file::flags::O_WRONLY {
            context.contents.lock().clear();
        }
        Ok(shared.clone())
    }

    fn read( shared: RefBorrow<'_, Device>, file: &File,
        data: &mut impl IoBufferWriter, offset: u64 ) -> Result<usize> {
        pr_info!("rust_mymem (read)\n");
        let buffer = shared.buffer.lock();

        if data.is_empty() {
            return Ok(0);
        }

        let offset: usize = offset as usize;
        let num_bytes: usize = data.len();

        // Write starting from offset
        data.write_slice(&buffer[offset..][..num_bytes])?;

        Ok(num_bytes)
    }

    fn write( shared: RefBorrow<'_, Device>, _: &File,
        data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        if data.is_empty() {
            return Ok(0);
        }
        let mut buffer = shared.buffer.lock();
        let num_bytes: usize = data.len();
        let offset: usize = offset as usize;
        data.read_slice(&mut buffer[offset..][..num_bytes])?;
        Ok(num_bytes)
    }

    fn seek( shared: RefBorrow<'_, Device>, _file: &File,
        _offset: SeekFrom) -> Result<u64> {
        pr_info!("rust_mymem (seek)\n");
        Ok(0)
    }
}
