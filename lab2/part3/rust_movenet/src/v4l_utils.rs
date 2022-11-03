use libc;
use std::str;
use std::ptr;
use std::fs::File;
use std::io::Write;
use std::mem::MaybeUninit;
use std::mem::zeroed;
use nix::{sys::ioctl, ioctl_read, ioctl_readwrite, ioctl_write_ptr};
use std::os::unix::io::RawFd;

const VIDIOC_QUERYCAP_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYCAP_TYPE_MODE: u8 = 0;

const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
const V4L2_MEMORY_MMAP: u32 = 1;
const VIDIOC_G_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_G_FMT_TYPE_MODE: u8 = 4;

const VIDIOC_S_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_S_FMT_TYPE_MODE: u8 = 5;

const VIDIOC_ENUM_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_ENUM_FMT_TYPE_MODE: u8 = 2;

const VIDIOC_REQBUFS_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_REQBUFS_TYPE_MODE: u8 = 8;
    
const VIDIOC_QUERYBUF_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYBUF_TYPE_MODE: u8 = 9;

const VIDIOC_STREAMON_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_STREAMON_TYPE_MODE: u8 = 18;
const VIDIOC_STREAMOFF_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_STREAMOFF_TYPE_MODE: u8 = 19;

const VIDIOC_QBUF_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_QBUF_TYPE_MODE: u8 = 15;
const VIDIOC_DQBUF_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_DQBUF_TYPE_MODE: u8 = 17;

#[repr(C)]
#[derive(Default)]
pub struct v4l2_capability {
    pub driver: [u8; 16],
    pub card: [u8; 32],
    pub bus_info: [u8; 32],
    pub version: u32,
    pub capabilities: u32,
    pub device_caps: u32,
    pub reserved: [u32; 3],
} 

#[repr(C)]
pub struct v4l2_format {
    my_type: u32,
    align: u32,
    width: u32,
    height: u32,
    pixelformat: u32,
    field: u32,
    bytesperline: u32,
    sizeimage: u32,
    others: [u8; 208 - 8*4],
}

#[repr(C)]
pub struct v4l2_fmtdesc {
    index: u32,
    my_type: u32,
    flags: u32,
    description: [u8; 32],
    pixelformat: u32,
    others: [u32; 4]
}


pub fn get_info(media_fd: &RawFd) {
    ioctl_read!(vidioc_querycap, VIDIOC_QUERYCAP_MAGIC, VIDIOC_QUERYCAP_TYPE_MODE, v4l2_capability);
    let mut info: v4l2_capability =  Default::default();
    match unsafe { vidioc_querycap(*media_fd, &mut info as *mut v4l2_capability) } {
        Ok(_) => {
            println!("get info [OK]");
            println!("driver: {:?}", str::from_utf8(&info.driver));
            println!("supports video capture: {:?}", info.capabilities & 0x1);
            println!("supports streaming: {:?}", info.capabilities & 0x04000000);
        },
        Err(e) => {
            println!("get info [FAILED]: {:?}", e);
        },
    }
}

pub fn get_format(media_fd: &RawFd)  {
    let mut format: v4l2_format = v4l2_format { 
        my_type: V4L2_BUF_TYPE_VIDEO_CAPTURE,
        align: 0,
        width: 0,
        height: 0,
        pixelformat: 0,
        field: 0,
        bytesperline: 0,
        sizeimage: 0,
        others: [0; 208-8*4],
    };

    ioctl_readwrite!(vidioc_g_fmt, VIDIOC_G_FMT_MAGIC, VIDIOC_G_FMT_TYPE_MODE, v4l2_format);
    match unsafe { vidioc_g_fmt(*media_fd, &mut format as *mut v4l2_format) } {
        Ok(_) => {
            println!("get fmt [OK]");
            println!("Image format:\n");
            println!("\ttype: {:?}\n", format.my_type);
            println!("\twidth: {:?}\n", format.width);
            println!("\theight: {:?}\n", format.height);
            println!("\tpixelformat: {:?}\n", format.pixelformat);
            println!("\tfield: {:?}\n", format.field);
            println!("\tbytesperline: {:?}\n", format.bytesperline);
            println!("\tsizeimage: {:?}\n", format.sizeimage);
        },
        Err(e) => {
            println!("get fmt [FAILED]: {:?}", e);
        },
    }
}

pub fn set_fmt_YUV422(media_fd: &RawFd) {
    let mut format: v4l2_format = v4l2_format { 
        my_type: V4L2_BUF_TYPE_VIDEO_CAPTURE,
        align: 0,
        width: 0,
        height: 0,
        pixelformat: 0,
        field: 0,
        bytesperline: 0,
        sizeimage: 0,
        others: [0; 208-8*4],
    };

    ioctl_readwrite!(vidioc_g_fmt, VIDIOC_G_FMT_MAGIC, VIDIOC_G_FMT_TYPE_MODE, v4l2_format);
    match unsafe { vidioc_g_fmt(*media_fd, &mut format as *mut v4l2_format) } {
        Ok(_) => (),
        Err(e) => {
            println!("get fmt [FAILED]: {:?}", e);
        },
    }

    format.pixelformat = 0x56595559;

    ioctl_readwrite!(vidioc_s_fmt, VIDIOC_S_FMT_MAGIC, VIDIOC_S_FMT_TYPE_MODE, v4l2_format);
    match unsafe { vidioc_s_fmt(*media_fd, &mut format as *mut v4l2_format) } {
        Ok(_) => {
            // success
            println!("set fmt [OK]");
            match unsafe { vidioc_g_fmt(*media_fd, &mut format as *mut v4l2_format) } {
                Ok(_) => {
                    println!("new pixelformat: {:?}\n", format.pixelformat);
                },
                Err(e) => {
                    println!("get fmt [FAILED]: {:?}", e);
                },
            }
        },
        Err(e) => {
            // failure
        },
    }
}

pub fn list_fmts(media_fd: &RawFd) {
   let mut fmtdesc = v4l2_fmtdesc {
        index: 0,
        my_type: 0,
        flags: 0,
        description: [0; 32],
        pixelformat: 0,
        others: [0; 4],
    };

    println!("\nListing supported formats:");
    fmtdesc.my_type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    ioctl_readwrite!(vidioc_enum_fmt, VIDIOC_ENUM_FMT_MAGIC, VIDIOC_ENUM_FMT_TYPE_MODE, v4l2_fmtdesc);
    loop {
        match unsafe { vidioc_enum_fmt(*media_fd, &mut fmtdesc as *mut v4l2_fmtdesc) } {
            Ok(ret) => {
                if ret != 0 {break;}
                println!("pixelformat: {:?}", fmtdesc.pixelformat);
                println!("desc: {:?}\n", str::from_utf8(&fmtdesc.description));
                fmtdesc.index+=1;
            },
            Err(e) => {
                break;
            },
        }
    }
}

#[repr(C)]
pub struct v4l2_requestbuffers {
    pub count: u32,
    pub my_type: u32,
    pub memory: u32,
    pub reserved: [u32; 2]
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

//#[derive(Default)]
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

impl Default for v4l2_buffer {
    fn default() -> v4l2_buffer { v4l2_buffer {
        index: 0,
        my_type: V4L2_BUF_TYPE_VIDEO_CAPTURE,
        bytesused: 0,
        flags: 0,
        field: 0,
        align: 0,
        timestamp: timeval { tv_sec: 0, tv_usec: 0 },
        timecode: v4l2_timecode {
            my_type: 0,
            flags: 0,
            frames: 0,
            seconds: 0,
            minutes: 0,
            hours: 0,
            userbits: [0; 4],
        },
        sequence: 0,
        memory: V4L2_MEMORY_MMAP,
        offset: 0,
        offset2: 0,
        length: 0,
        reserved1: 0,
        reserved2: [0; 2],
    }
    }
}


pub struct buffer {
    pub start: * mut libc::c_void,
    pub length: usize
}

pub fn request_buffer(media_fd: &RawFd, reqbuf: &mut v4l2_requestbuffers) {
    reqbuf.my_type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    reqbuf.memory = V4L2_MEMORY_MMAP;
    reqbuf.count = 1; // REQBUFS gives me 2 buffers no matter what this value is, so I am just going to use one buffer

    ioctl_readwrite!(vidioc_reqbufs, VIDIOC_REQBUFS_FMT_MAGIC, VIDIOC_REQBUFS_TYPE_MODE, v4l2_requestbuffers);
    match unsafe { vidioc_reqbufs(*media_fd, reqbuf as *mut v4l2_requestbuffers) } {
        Ok(_) => {
            println!("got {:?} buffers", reqbuf.count);
        },
        Err(e) => {
            println!("requesting buffers [FAILED]: {:?}", e);
        }
    }
}

pub fn map_buffer(media_fd: &RawFd, resbuf: &mut buffer, reqbuf: &v4l2_requestbuffers) {
    let mut buffer: v4l2_buffer = Default::default();

    ioctl_readwrite!(vidioc_querybuf, VIDIOC_QUERYBUF_FMT_MAGIC, VIDIOC_QUERYBUF_TYPE_MODE, v4l2_buffer);
    match unsafe { vidioc_querybuf(*media_fd, &mut buffer as *mut v4l2_buffer) } {
        Ok(_) => (),
        Err(e) => {
            println!("querying buffer [FAILED]: {:?}", e);
        },
    }

    resbuf.length = buffer.length as usize; /* remember for munmap() */
    println!("{:?}", resbuf.length);

    resbuf.start = unsafe {
        libc::mmap( ptr::null_mut(), resbuf.length,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            *media_fd, buffer.offset.into())
    };
    if resbuf.start == libc::MAP_FAILED {
        println!("mmap failed" );
    }
}

pub fn start_streaming(media_fd: &RawFd) {
    // Activate streaming
    let mut my_type = V4L2_BUF_TYPE_VIDEO_CAPTURE as i32;

    ioctl_write_ptr!(vidioc_streamon, VIDIOC_STREAMON_FMT_MAGIC, VIDIOC_STREAMON_TYPE_MODE, i32);
    match unsafe { vidioc_streamon(*media_fd, &mut my_type as *mut i32) } {
        Ok(_) => (),
        Err(e) => {
            println!("stream on [FAILED]: {:?}", e);
        },
    }
}

 
pub fn stop_streaming(media_fd: &RawFd) {
    // Activate streaming
    let mut my_type = V4L2_BUF_TYPE_VIDEO_CAPTURE as i32;

    ioctl_write_ptr!(vidioc_streamoff, VIDIOC_STREAMOFF_FMT_MAGIC, VIDIOC_STREAMOFF_TYPE_MODE, i32);
    match unsafe { vidioc_streamoff(*media_fd, &mut my_type as *mut i32) } {
        Ok(_) => (),
        Err(e) => {
            println!("stream on [FAILED]: {:?}", e);
        },
    }
}

pub fn queue_buffer(media_fd: &RawFd, qbuffer: &mut v4l2_buffer) {
    ioctl_readwrite!(vidioc_qbuf, VIDIOC_QBUF_FMT_MAGIC, VIDIOC_QBUF_TYPE_MODE, v4l2_buffer);
    match unsafe { vidioc_qbuf(*media_fd, qbuffer as *mut v4l2_buffer) } {
        Ok(_) => (),
        Err(e) => {
            println!("queue buf [FAILED]: {:?}", e);
        },
    }
}


pub fn dequeue_buffer(media_fd: &RawFd, qbuffer: &mut v4l2_buffer) {
    ioctl_readwrite!(vidioc_dqbuf, VIDIOC_DQBUF_FMT_MAGIC, VIDIOC_DQBUF_TYPE_MODE, v4l2_buffer);
    match unsafe { vidioc_dqbuf(*media_fd, qbuffer as *mut v4l2_buffer) } {
        Ok(_) => (),
        Err(e) => {
            println!("dequeue buf [FAILED]: {:?}", e);
        },
    }
}

pub fn buffer_to_bytes(resbuf: &buffer, bytes: &mut Vec<u8>) {
    unsafe { 
        ptr::copy_nonoverlapping(bytes.as_ptr(), resbuf.start as *mut u8, resbuf.length);
    }
}

pub fn save_yuv(mut name: String, buffer: &[u8]) {
    name.push_str(".yuv");
    let mut file = File::create(name).unwrap();
    file.write_all(buffer);
}

pub fn destroy_buffer(resbuf: &mut buffer) {
    unsafe { libc::munmap(resbuf.start, resbuf.length); }; 
}
