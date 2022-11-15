use kernel::prelude::*;
use kernel::{
    delay::coarse_sleep,
    c_str,
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    task::Task,
    sync::{smutex::Mutex, Ref, RefBorrow},
    net::{Ipv4Addr, init_ns, TcpStream},
};
use kernel::bindings;
use core::mem;
use core::marker;
use core::ptr;
use core::ffi;
use core::default::Default;
use core::time::Duration;

// constants obtained by printing out values in C
const VIDIOC_STREAMON: u32 = 1074026002;
const VIDIOC_STREAMOFF: u32 = 1074026003;
const VIDIOC_QBUF: u32 = 3227014671; 
const VIDIOC_DQBUF: u32 = 3227014673;

const IM_SIZE: usize = 118784;
const PAGESIZE: usize = 4096;

module! {
    type: RustCamera,
    name: "rust_camera",
    author: "Evan Gerritz",
    description: "A simple module that reads camera input.",
    license: "GPL",
}

const OUT_BUF_SIZE: usize = 17*3*4;

kernel::init_static_sync! {
    static user_msg: Mutex<kernel_msg> = kernel_msg {
        start_pfn: 0, num_pfns: 0, my_type: 0, buffer:0
    };
    static shared_output: Mutex<[u8; OUT_BUF_SIZE]> = [0u8; OUT_BUF_SIZE];
}


struct RustCamera {
    _dev: Pin<Box<miscdev::Registration<RustCamera>>>,
}

struct Device {
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
    unsafe { (pfn << PAGE_SHIFT) + bindings::page_offset_base }
}

impl kernel::Module for RustCamera {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("RustCamera (init)\n");

        // make RustCamera a miscdev as you have done in A1P4
        let state = Ref::try_new( Device {})?;

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

    // sends data in shared_output to user
    fn read( shared: RefBorrow<'_, Device>, _file: &File,
        data: &mut impl IoBufferWriter, offset: u64 ) -> Result<usize> {
        let mut output = shared_output.lock();

        let num_bytes: usize = data.len();

        data.write_slice(&*output)?;
        Ok(num_bytes)
            
    }

    // gets the start message from the user program, launches thread to 
    // start capture loop
    fn write( shared: RefBorrow<'_, Device>, _: &File,
        data: &mut impl IoBufferReader, _offset: u64) -> Result<usize> {
        // get userspace data
        pr_info!("RustCamera (write)\n");
        let mut msg_bytes = [0u8; 32];
        data.read_slice(&mut msg_bytes).expect("couldn't read data");
        {
            // put in block so it will give up the lock on user_msg
            let mut my_msg = user_msg.lock();
            *my_msg = unsafe { mem::transmute::<[u8; 32], kernel_msg>(msg_bytes) }; 
        }
        Task::spawn(fmt!(""), move || {
            start_capture(); 
        }).expect("couldn't start task");
        //start_capture(shared); // user program will block here indefinitely
        Ok(0)
    }
}

// this is the main capture loop
fn start_capture() {
    // open camera
    let fname = c_str!("/dev/video2");
    let mut camera_filp = unsafe { bindings::filp_open(fname.as_ptr() as *const i8, (bindings::O_RDWR | bindings::O_NONBLOCK) as i32, 0) };
    if camera_filp < 0x100 as *mut _ {
        pr_info!("filp_open failed!");
        return;
    } else {
        pr_info!("file name: {:?}", core::str::from_utf8(&(*(*camera_filp).f_path.dentry).d_iname));
    }

    let mut socket = ptr::null_mut();
    let ret = unsafe {
        bindings::sock_create_kern(
            &mut bindings::init_net,
            bindings::PF_INET as ffi::c_int, 
            bindings::sock_type_SOCK_STREAM as ffi::c_int,
            bindings::IPPROTO_TCP as ffi::c_int,
            &mut socket
    )};
    pr_info!("sock create ret: {:?}\n", ret);

    let mut saddr: bindings::sockaddr_in = Default::default();
    saddr.sin_family = bindings::PF_INET as u16;
    saddr.sin_port = 0x901f; // 8080 -> 0x1f90 -> 0x901f
    saddr.sin_addr.s_addr = 0x100007f; // 127.0.0.1 -> 0x7f00001 -> big endian

    let mut saddr: bindings::sockaddr = unsafe { mem::transmute::<bindings::sockaddr_in, bindings::sockaddr>(saddr) };
    let ret = unsafe { bindings::kernel_connect(socket, &mut saddr,
            mem::size_of::<bindings::sockaddr_in>().try_into().unwrap(),
            (bindings::_IOC_READ | bindings::_IOC_WRITE) as ffi::c_int) };
    pr_info!("sock connect ret: {:?}\n", ret);

    
    // changed sock from pub(crate) to pub in linux/rust/kernel/net.rs
    let stream = TcpStream { sock: socket };

    let msg = &*user_msg.lock();
    pr_info!("{:?}\n", msg.buffer);
    // only do it 10000 times
    for i in 0..10000 {
        let mut pfn = msg.start_pfn;
        queue_buffer(camera_filp, msg.buffer);
        // send each page in its own chunk
        for i in 0..29{
            let buffer_kaddr = pfn_to_kaddr(pfn);    
            let buffer_p = unsafe { mem::transmute::<u64, *mut [u8; PAGESIZE]>(buffer_kaddr) } ;
            stream.write(& unsafe { *buffer_p }, true).expect("could not send bytes in buffer_p");
            pfn += 1; // pfns are always (at least empirically) sequential
        }
        { // receive the output and put in output buffer
            // in block so we will give up the lock after stream.read, so read can get the output
            let mut output = &mut *shared_output.lock();
            stream.read(output, true).expect("could not receive bytes in buffer");
        }
        dequeue_buffer(camera_filp, msg.buffer);
        coarse_sleep(Duration::from_millis(25));
    }
}

fn queue_buffer(camera_f: *mut bindings::file, buffer: u64) {
    let r = unsafe { bindings::vfs_ioctl(camera_f, VIDIOC_QBUF, buffer) }; 
    if r < 0 {
        pr_info!("qbuf failed with {:?}\n", r);
    } else {
        //pr_info!("qbuf success\n");
    }
}

fn dequeue_buffer(camera_f: *mut bindings::file, buffer: u64) {
    let r = unsafe { bindings::vfs_ioctl(camera_f, VIDIOC_DQBUF, buffer) }; 
    if r < 0 {
        pr_info!("dqbuf failed with {:?}\n", r);
    } else {
        //pr_info!("dqbuf success\n");
    }
}
