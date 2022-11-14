use opencv::highgui::imshow;
use opencv::highgui::wait_key;
use opencv::core::*;

use libc;
use std::{fs::File, os::unix::prelude::AsRawFd, str, ptr, thread, io, mem};
use std::io::{Read, Write};
use std::time::{Instant, Duration};
use nix::{sys::ioctl, ioctl_read, ioctl_readwrite};

mod utils;
use utils::*;

mod v4l_utils;
use v4l_utils::*;

mod client;
use client::Server;
use std::process;

struct kernel_msg {
    start_pfn: u64,
    num_pfns: u64,
    my_type: *const i32,
    buffer: *const v4l2_buffer,
}

fn main() {
    println!("My pid is {}", process::id());
    let debug = false;
    let mut file = File::options()
		.write(true)
		.read(true)
		.open("/dev/video2")
		.unwrap();

    let mut my_pagemap = File::open("/proc/self/pagemap").expect("couldn't open pagemap");
    let mut camera_module = File::options().write(true).read(true)
		.open("/dev/rust_camera").expect("couldn't open camera module");

    let media_fd = file.as_raw_fd();
    if debug {
        println!("camera fd = {}", media_fd);
    }

    if debug {
        get_info(&media_fd);
        get_format(&media_fd);
        list_fmts(&media_fd);
    }
    set_fmt_YUV422(&media_fd);

    let mut resbuf = buffer {
        start: ptr::null_mut(),
        length: 0
    };

    let mut reqbuf: v4l2_requestbuffers = Default::default();

    request_buffer(&media_fd, &mut reqbuf);

    map_buffer(&media_fd, &mut resbuf, &reqbuf);
    let mut va: u64 = unsafe{&(*resbuf.start) as *const _ as u64};
    let start_pfn = va_to_pfn(&mut my_pagemap, va);
    let num_pfns = resbuf.length as u64 / PAGESIZE;
    let my_type = V4L2_BUF_TYPE_VIDEO_CAPTURE as i32;
    let mut qbuffer: v4l2_buffer = Default::default();
    
    let msg = kernel_msg {
        start_pfn, num_pfns, my_type: &my_type as *const i32, buffer: &qbuffer as *const v4l2_buffer,
    };
    println!("{:?} {:?} {:?} {:?}", msg.start_pfn, msg.num_pfns, msg.my_type, msg.buffer);
    let msg_bytes = unsafe { mem::transmute::<kernel_msg, [u8; 32]>(msg)};

    start_streaming(&media_fd);

    // tell camera module to start capturing
    camera_module.write(&msg_bytes).expect("couldn't write");

    const IMG_WIDTH: i32 = 320;
    const IMG_HEIGHT: i32 = 180;    

    const LEN_OUTPUT: usize = 17*3; 

    thread::sleep(Duration::from_millis(2000));
    let now = Instant::now();
    let mut count: i32 = 0;
    // wait for kernel to get ready
    loop {
        let mut output_bytes: [u8; LEN_OUTPUT*4] = [0; LEN_OUTPUT*4];
        let mut output: [f32; LEN_OUTPUT] = [0.0; LEN_OUTPUT];
    
        // create blank matrix to show points on
        // needs to be created each time or else points accumulate
        let mut blank = Mat::new_size_with_default(
            Size{ height: IMG_HEIGHT, width: IMG_WIDTH },
            CV_8UC3, (255.0,255.0,255.0,0.0).into()
        ).unwrap();

        // wait to poll from kernel module
        thread::sleep(Duration::from_millis(25));

        camera_module.read(&mut output_bytes); 
        client::bytes_to_f32(&output_bytes, &mut output);
        if output[0] != 0.0 {
            draw_keypoints(&mut blank, &output, 0.25);
            imshow("MoveNet", &blank).expect("imshow [ERROR]");
        }
        count += 1;

		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			break;
        }
    }

    // calculate and print fps
    let total_time = now.elapsed();
    let total_time: f64 = (total_time.as_secs() as f64) + (total_time.subsec_nanos() as f64) / 1e9;
    println!("fps: {:?} ", (count as f64)/total_time);

    // clean up
    stop_streaming(&media_fd);
    destroy_buffer(&mut resbuf);
}
