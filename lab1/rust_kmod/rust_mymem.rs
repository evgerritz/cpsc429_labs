// SPDX-License-Identifier: GPL-2.0

//! Rust miscellaneous device sample.

use kernel::prelude::*;
use kernel::{
    /file::{self, File, SeekFrom},
    //io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{smutex::Mutex, Ref},
};

module! {
    type: RustMymem,
    name: "rust_mymem",
    author: "Evan Gerritz",
    description: "mymem test module in Rust",
    license: "GPL",
}

const BUFFER_SIZE: usize = 512*1024;

pub struct RustMymem;

static BUFFER: Mutex<[u8; BUFFER_SIZE]> = Mutex::new( [0u8; BUFFER_SIZE] );

impl kernel::Module for RustMymem {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

        let buffer_p = &BUFFER.lock();

        pr_info!("buffer len: {:?}", buffer_p.len());
        Ok(RustMymem)
    }
}

impl Drop for RustMymem {
    fn drop(&mut self) {
        pr_info!("rust_mymem (exit)\n");
    }
}

impl RustMymem {
    pub fn read( &mut self, outbuf: &mut [u8], offset: usize ) -> usize {
        pr_info!("rust_mymem (read)");
        /*if data.is_empty() {
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

        *offset_p += num_bytes; */

        0
    }

    pub fn write( &mut self, inbuf: &[u8], offset: usize ) -> usize {
        pr_info!("rust_mymem (write)");
        0
        /*if data.is_empty() {
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
        Ok(num_bytes) */
    }
}

