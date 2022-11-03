//use v4l::buffer::Type;
//use v4l::device::Device;
use opencv::highgui::imshow;

use std::{fs::File, os::unix::prelude::AsRawFd, str, ptr};
use nix::{sys::ioctl, ioctl_read, ioctl_readwrite};
use std::mem::size_of;

mod utils;
use utils::*;

mod v4l_utils;
use v4l_utils::*;

mod client;
use client::Server;

use std::time::Instant;
use std::time::Duration;
use std::thread;

fn main() {
    let mut file = File::options()
		.write(true)
		.read(true)
		.open("/dev/video2")
		.unwrap();

    let media_fd = file.as_raw_fd();
    println!("camera fd = {}", media_fd);

    get_info(&media_fd);
    get_format(&media_fd);
    set_fmt_YUV422(&media_fd);
    list_fmts(&media_fd);

    let mut resbuf = buffer {
        start: ptr::null_mut(),
        length: 0
    };

    let mut reqbuf = v4l2_requestbuffers {
        count: 0,
        my_type: 0,
        memory: 0,
        reserved: [0; 2]
    };
    
    request_buffer(&media_fd, &mut reqbuf);

    map_buffer(&media_fd, &mut resbuf, &reqbuf);
    
    let mut qbuffer: v4l2_buffer = Default::default();
    queue_buffer(&media_fd, &mut qbuffer);
    start_streaming(&media_fd);

    thread::sleep(Duration::from_millis(500));

    let mut bytes = vec![0; 118784];
    let name: String = String::from("output");
    buffer_to_bytes(&resbuf, &mut bytes);
    save_yuv(name, &bytes);

    dequeue_buffer(&media_fd, &mut qbuffer);
    stop_streaming(&media_fd);
    destroy_buffer(&mut resbuf);
}

/*fn main() {


    // establish connection to server
    let mut server = Server::new();
    const LEN_OUTPUT: usize = 17*3; 

    let mut counter: u64 = 0;
    let mut total_time: Duration = Duration::new(0, 0);

	loop {
        let now = Instant::now();
		let mut frame = Mat::default();
		cam.read(&mut frame).expect("VideoCapture: read [FAILED]");

		if frame.size().unwrap().width > 0 {
            // flip the image horizontally
            let mut flipped = Mat::default();
            flip(&frame, &mut flipped, 1).expect("flip [FAILED]");

            let image_bytes: Vec<u8> = client::image_to_bytes(&flipped);
            let mut output_bytes: [u8; LEN_OUTPUT*4] = [0; LEN_OUTPUT*4];
            let mut output: [f32; LEN_OUTPUT] = [0.0; LEN_OUTPUT];

            //println!("{:?}", image_bytes.len());
            server.send_bytes(&image_bytes);
            server.receive_bytes(&mut output_bytes); 

            client::bytes_to_f32(&output_bytes, &mut output);
			draw_keypoints(&mut flipped, &output, 0.25);
			imshow("MoveNet", &flipped).expect("imshow [ERROR]");

            counter += 1;
		}
        total_time += now.elapsed();
		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			break;
		}
	}
    println!("avg images processed/sec: {:?} ", total_time/(counter as u32));
}*/

