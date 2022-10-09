// SPDX-License-Identifier: GPL-2.0

//! Rust Mymem module, part 5 version

use kernel::prelude::*;
use kernel::{
    //file::{self, File, SeekFrom},
    //io_buffer::{IoBufferReader, IoBufferWriter},
    //miscdev,
    sync::{smutex::Mutex}//, Ref},
};

module! {
    type: RustMymem,
    name: "rust_mymem",
    author: "Evan Gerritz",
    description: "mymem test module in Rust",
    license: "GPL",
}

const BUFFER_SIZE: usize = 512*1024;

/// struct providing accessing to the module for our test program
pub struct RustMymem;

static BUFFER: Mutex<[u8; BUFFER_SIZE]> = Mutex::new( [0u8; BUFFER_SIZE] );

impl kernel::Module for RustMymem {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_mymem (init)\n");

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
    /// reads into the buffer, starting at offset
    pub fn read( &mut self, outbuf: &mut [u8], offset: usize ) -> usize {
        let buffer_p = &BUFFER.lock();

        let mut num_bytes: usize = outbuf.len();
        let max_bytes: usize = buffer.len() - offset;
        if max_bytes < num_bytes {
            num_bytes = max_bytes; 
        }

        if num_bytes + offset > BUFFER_SIZE {
            return Err(EINVAL);
        }
        // Write starting from offset
        outbuf[..].clone_from_slice(&(*buffer_p[offset..][..num_bytes]))

        num_bytes
    }

    /// writes to the buffer, starting at offset
    pub fn write( &mut self, inbuf: &[u8], offset: usize ) -> usize {

        let mut buffer_p = &BUFFER.lock();

        let num_bytes: usize = inbuf.len();

        if num_bytes + offset > BUFFER_SIZE {
            return Err(EINVAL);
        }

        (*buffer_p[offset..][..num_bytes]).clone_from_slice(&inbuf);

        num_bytes 
    }
}

