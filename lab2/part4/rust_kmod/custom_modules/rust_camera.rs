use kernel::prelude::*;
use kernel::{
    delay::coarse_sleep,
    c_str,
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{smutex::Mutex, Ref, RefBorrow},
};
use kernel::bindings;
use core::mem;
use core::time::Duration;

// constants obtained by printing out values in C
const VIDIOC_STREAMON: u32 = 1074026002;
const VIDIOC_STREAMOFF: u32 = 1074026003;
const VIDIOC_QBUF: u32 = 3227014671; 
const VIDIOC_DQBUF: u32 = 3227014673;

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
    my_type: u64,
    buffer: u64
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

const PAGE_SHIFT: u64 = 12;

fn pfn_to_kaddr(pfn: u64) -> u64{
    (pfn << 12) + bindings::page_offset_base
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
        // get userspace data
        pr_info!("RustCamera (write)\n");
        let mut msg_bytes = [0u8; 32];
        data.read_slice(&mut msg_bytes).expect("couldn't read data");
        let msg: kernel_msg = unsafe { mem::transmute::<[u8; 32], kernel_msg>(msg_bytes) };

        let fname = c_str!("/dev/video2");
        let mut camera_filp = unsafe { bindings::filp_open(fname.as_ptr() as *const i8, bindings::O_RDWR as i32, 0) };

        pr_info!("page offset: {:?}", bindings::page_offset_base);
        pr_info!("kaddr: {:?}", pfn_to_kaddr(358387));

        queue_buffer(camera_filp, msg.buffer);
        start_streaming(camera_filp, msg.my_type);
        loop {
            queue_buffer(camera_filp, msg.buffer);
            coarse_sleep(Duration::from_millis(2));
            dequeue_buffer(camera_filp, msg.buffer);
        }
        stop_streaming(camera_filp, msg.my_type);
        Ok(0)
    }
}

fn start_streaming(camera_f: *mut bindings::file, my_type: u64) {
    // Activate streaming
    if unsafe { bindings::vfs_ioctl(camera_f, VIDIOC_STREAMON, my_type) } < 0 {
        pr_info!("streamon failed!");
    }
}

fn stop_streaming(camera_f: *mut bindings::file, my_type: u64) {
    if unsafe { bindings::vfs_ioctl(camera_f, VIDIOC_STREAMOFF, my_type) } < 0 {
        pr_info!("streamoff failed!");
    }
}

fn queue_buffer(camera_f: *mut bindings::file, buffer: u64) {
    if unsafe { bindings::vfs_ioctl(camera_f, VIDIOC_QBUF, buffer) } < 0 {
        pr_info!("qbuf failed!");
    }
}

fn dequeue_buffer(camera_f: *mut bindings::file, buffer: u64) {
    if unsafe { bindings::vfs_ioctl(camera_f, VIDIOC_DQBUF, buffer) } < 0 {
        pr_info!("dqbuf failed!");
    }
}
