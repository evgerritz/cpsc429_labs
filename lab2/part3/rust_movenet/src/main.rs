use opencv::highgui::imshow;
use opencv::highgui::wait_key;
use opencv::core::*;
//use opencv::imgcodecs::{imdecode, IMREAD_COLOR};

use libc;
use std::{fs::File, os::unix::prelude::AsRawFd, str, ptr, thread};
use std::time::{Instant, Duration};
use nix::{sys::ioctl, ioctl_read, ioctl_readwrite};

mod utils;
use utils::*;

mod v4l_utils;
use v4l_utils::*;

mod client;
use client::Server;


fn main() {
    let debug = false;
    let mut file = File::options()
		.write(true)
		.read(true)
		.open("/dev/video2")
		.unwrap();

    let media_fd = file.as_raw_fd();
    if debug {
        println!("camera fd = {}", media_fd);
    }

    // connect to server
    let mut server = Server::new();


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
    
    let mut qbuffer: v4l2_buffer = Default::default();
    queue_buffer(&media_fd, &mut qbuffer);
    start_streaming(&media_fd);

    const IMG_WIDTH: i32 = 320;
    const IMG_HEIGHT: i32 = 180;

    const LEN_INPUT: usize = 118784;
    const LEN_OUTPUT: usize = 17*3; 
    let now = Instant::now();
    let i = 0;
    loop {
        // create blank matrix to show points on
        // needs to be created each time or else points accumulate
        let mut blank = Mat::new_size_with_default(
            Size{ height: IMG_HEIGHT, width: IMG_WIDTH },
            CV_8UC3, (255.0,255.0,255.0,0.0).into()
        ).unwrap();

        queue_buffer(&media_fd, &mut qbuffer);
        thread::sleep(Duration::from_millis(25));

        let mut bytes = vec![0u8; LEN_INPUT];
        buffer_to_bytes(&resbuf, &mut bytes);
        server.send_bytes(&bytes);

        let mut output_bytes = vec![0u8; LEN_OUTPUT*4];
        let mut output = [0.0f32; LEN_OUTPUT];
        server.receive_bytes(&mut output_bytes); 
        client::bytes_to_f32(&output_bytes, &mut output);
        draw_keypoints(&mut blank, &output, 0.25);
        imshow("MoveNet", &blank).expect("imshow [ERROR]");

        dequeue_buffer(&media_fd, &mut qbuffer);

		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			break;
        }
        // save for debugging
        if debug {
            let mut name: String = String::from("frames_test/");
            name.push_str(&i.to_string());
            save_yuv(name, &bytes);
        }
    }

    // calculate and print fps
    let total_time = now.elapsed();
    let total_time: f64 = (total_time.as_secs() as f64) + (total_time.subsec_nanos() as f64) / 1e9;
    println!("fps: {:?} ", 30.0/total_time);

    // clean up
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

