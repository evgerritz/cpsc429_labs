use kernel::prelude::*;
use kernel::sync::smutex::Mutex;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{smutex::Mutex, Ref, RefBorrow},
};


use kernel::bindings;

module! {
    type: RustCamera,
    name: "rust_camera",
    author: "Evan Gerritz",
    description: "A simple module that reads camera input.",
    license: "GPL",
}

const OUT_BUF_SIZE: usize = 17*3;

struct RustCamera {
    _dev: Pin<Box<miscdev::Registration<RustMymem>>>,
}

struct Device {
    output: Mutex<[u8; OUT_BUF_SIZE]>,
}


impl kernel::Module for RustCamera {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("RustCamera (init)\n");

        // make RustCamera a miscdev as you have done in A1P4
        let state = Ref::try_new( Device {
            buffer: Mutex::new([0u8; BUFFER_SIZE]),
        })?;

        Ok(RustMymem {                  // 438 == 0o666
            _dev: miscdev::Options::new().mode(438).register_new(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustCamera {
    fn drop(&mut self) {
        pr_info!("RustCamera (exit)\n");
    }
}

#[vtable]
impl file::Operations for RustCamera {
    type OpenData = Ref<Device>;
    type Data = Ref<Device>;

    fn open(shared: &Ref<Device>, _file: &File) -> Result<Self::Data> {
        pr_info!("rust_camera (open)\n");
        Ok(shared.clone())
    }

    fn read( shared: RefBorrow<'_, Device>, _file: &File,
        data: &mut impl IoBufferWriter, offset: u64 ) -> Result<usize> {
        let buffer = shared.buffer.lock();

        if data.is_empty() {
            return Ok(0);
        }

        let mut num_bytes: usize = data.len();
        let max_bytes: usize = buffer.len();
        if max_bytes < num_bytes {
            num_bytes = max_bytes; 
        }

        if num_bytes > BUFFER_SIZE {
            return Err(EINVAL);
        }
        // Write starting from offset
        data.write_slice(&buffer[..num_bytes])?;

        Ok(num_bytes)
    }

    fn write( shared: RefBorrow<'_, Device>, _: &File,
        data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        if data.is_empty() {
            return Ok(0);
        }

        let mut buffer = shared.buffer.lock();

        let num_bytes: usize = data.len();

        let new_len = num_bytes;
        if new_len > BUFFER_SIZE {
            return Err(EINVAL);
        }

        data.read_slice(&mut buffer[..num_bytes])?;
        Ok(num_bytes)
    }
}
