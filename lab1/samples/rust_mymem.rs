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
    buffer: Box<[u8; BUFFER_SIZE]>
}

impl kernel::Module for RustMymem {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

        let state = Ref::try_new(SharedState {
            buffer: Box::new([0; BUFFER_SIZE]),
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

const BUFFER_SIZE: usize = 512*1024;

#[vtable]
impl file::Operations for RustMymem {
    type OpenData = Ref<SharedState>;
    type Data = Ref<SharedState>;

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

        let num_bytes: usize = data.len();
        pr_info!("num bytes: {:?}", num_bytes);

        // Write starting from offset
        let start: usize = offset as usize;
        let stop: usize = num_bytes + offset as usize;
        let mut buffer_slice: Vec<u8> = Vec::new();
        for i in start..stop {
            pr_info!("buffer: {:?}", buffer[i]);
            //buffer_slice.try_push(buffer[i])?;
        }

        //data.write_slice(&buffer_slice[..])?;

        Ok(data.len())
    }

    fn write( shared: RefBorrow<'_, SharedState>, _: &File,
        data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        if data.is_empty() {
            return Ok(0);
        }
        let mut buffer = shared.buffer;
        let num_bytes: usize = data.len();
        let to_write: Vec<u8>;
        to_write = data.read_all()?;
        for i in (offset as usize)..(offset as usize + num_bytes) {
            buffer[i] = to_write[i]; 
        }
        Ok(data.len())
    }

    fn seek( shared: RefBorrow<'_, SharedState>, _file: &File,
        _offset: SeekFrom) -> Result<u64> {
        pr_info!("rust_mymem (seek)\n");
        Ok(0)
    }
}
