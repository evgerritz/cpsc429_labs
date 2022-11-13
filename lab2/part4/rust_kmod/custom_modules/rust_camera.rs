use kernel::prelude::*;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{smutex::Mutex, Ref, RefBorrow},
};
use kernel::bindings;
use core::mem;

module! {
    type: RustCamera,
    name: "rust_camera",
    author: "Evan Gerritz",
    description: "A simple module that reads camera input.",
    license: "GPL",
}

const OUT_BUF_SIZE: usize = 17*3;

struct RustCamera {
    _dev: Pin<Box<miscdev::Registration<RustCamera>>>,
}

struct Device {
    output: Mutex<[u8; OUT_BUF_SIZE]>,
}

struct kernel_msg {
    start_pfn: u64,
    num_pfns: u64,
    my_type: *mut i32,
    buffer: *mut v4l2_buffer,
}

#[repr(C)]
struct timeval {
    tv_sec: u64,
    tv_usec: u64
}

#[repr(C)]
struct v4l2_timecode {
    my_type: u32,
    flags: u32,
    frames: u8,
    seconds: u8,
    minutes: u8,
    hours: u8,
    userbits: [u8; 4],
}

#[repr(C)]
pub struct v4l2_buffer {
    index: u32,
    my_type: u32,
    bytesused: u32,
    flags: u32,
    field: u32,
    align: u32,
    timestamp: timeval,
    timecode: v4l2_timecode,
    sequence: u32,
    memory: u32,
    offset: u32,
    offset2: u32,
    length: u32,
    reserved1: u32,
    reserved2: [u32; 2],
}


impl kernel::Module for RustCamera {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("RustCamera (init)\n");

        // make RustCamera a miscdev as you have done in A1P4
        let state = Ref::try_new( Device {
            output: Mutex::new([0u8; OUT_BUF_SIZE]),
        })?;

        Ok(RustCamera {                  // 438 == 0o666
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
        if data.is_empty() {
            return Ok(0);
        }

        let mut buffer = shared.output.lock();

        let num_bytes: usize = data.len();

        let new_len = num_bytes;
        if new_len > OUT_BUF_SIZE {
            return Err(EINVAL);
        }

        data.write_slice(&mut buffer[..num_bytes])?;
        Ok(num_bytes)
            
    }

    fn write( shared: RefBorrow<'_, Device>, _: &File,
        data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        let mut msg_bytes = [0u8; 32];
        data.read_slice(&mut msg_bytes);
        let msg: kernel_msg = unsafe { mem::transmute::<[u8; 32], kernel_msg>(msg_bytes) };
        pr_info!("{:?}\n", msg_bytes);
        Ok(0);
    }
}
