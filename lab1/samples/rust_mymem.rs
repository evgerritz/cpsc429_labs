// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use kernel::prelude::*;
use kernel::{
    file::{self, File, SeekFrom},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{Ref, RefBorrow,}//, CondVar, Mutex, UniqueRef},
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

struct SharedState {
    buffer: [u8; BUFFER_SIZE]
}

impl kernel::Module for RustMymem {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

        let state = Ref::try_new(SharedState {
            buffer: [0; BUFFER_SIZE],
        })?;

        Ok(RustMymem {
            _dev: miscdev::Options::new().mode(0666).register_new(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMymem {
    fn drop(&mut self) {
        pr_info!("rust_mymem (exit)\n");
    }
}

const BUFFER_SIZE: usize = 512*1024;

#[vtable]
impl file::Operations for RustMymem {
    type OpenData = Ref<SharedState>;
    type Data = Ref<SharedState>;
    //kernel::declare_file_operations!(read, write);

    fn open(shared: &Ref<SharedState>, _file: &File) -> Result<Self::Data> {
        pr_info!("rust_mymem (open)\n");
        Ok(shared.clone())
    }

    fn read( shared: RefBorrow<'_, SharedState>, file: &File,
        data: &mut impl IoBufferWriter, offset: u64 ) -> Result<usize> {
        pr_info!("rust_mymem (read)\n");
        // Succeed if the caller doesn't provide a buffer 
        if data.is_empty() {
            return Ok(0);
        }

        let buffer = shared.buffer;

        // Write a one-byte 1 to the reader.
        data.write_slice(&buffer[(offset as usize)..])?;
        Ok(1)
    }

    fn write( shared: RefBorrow<'_, SharedState>, _: &File,
        data: &mut impl IoBufferReader, _offset: u64) -> Result<usize> {
        pr_info!("rust_mymem (write)\n");

        Ok(data.len())
    }

    fn seek( shared: RefBorrow<'_, SharedState>, _file: &File,
        _offset: SeekFrom) -> Result<u64> {
        pr_info!("rust_mymem (seek)\n");
        Ok(0)
    }
}
