use opencv::highgui::imshow;
use opencv::highgui::wait_key;
use opencv::core::*;
use tflitec::interpreter::{Interpreter, Options};
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

    // start interpreter
	let options = Options::default();
	let path = format!("../splitter/output/upper.tflite");
	let interpreter = Interpreter::with_model_path(&path, Some(options)).unwrap();
	interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");

    // for testing lower network locally
    /*
	let options2 = Options::default();
	let path2 = format!("../splitter/output/lower.tflite");
	let interpreter2 = Interpreter::with_model_path(&path2, Some(options2)).unwrap();
	interpreter2.allocate_tensors().expect("Allocate tensors [FAILED]");
    */

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
    let mut count: i32 = 0;
    loop {
        // create blank matrix to show points on
        // needs to be created each time or else points accumulate
        let mut blank = Mat::new_size_with_default(
            Size{ height: IMG_HEIGHT, width: IMG_WIDTH },
            CV_8UC3, (255.0,255.0,255.0,0.0).into()
        ).unwrap();

        queue_buffer(&media_fd, &mut qbuffer);
        thread::sleep(Duration::from_millis(25));

        let mut image_bytes = vec![0u8; LEN_INPUT];
        buffer_to_bytes(&resbuf, &mut image_bytes);
        let mut rgb_bytes = vec![0; LEN_INPUT*3/2];
        yuv422_to_rgb24(&image_bytes, &mut rgb_bytes, IMG_WIDTH, IMG_HEIGHT);
        let rgb_mat = unsafe {
            Mat::new_rows_cols_with_data(
                IMG_HEIGHT, IMG_WIDTH, CV_8UC3,
                rgb_bytes.as_mut_ptr() as *mut libc::c_void,
                (IMG_WIDTH*3).try_into().unwrap()
            ).unwrap()
        };

        let resized_img = resize_with_padding(&rgb_mat, [192, 192]);
        let vec_2d: Vec<Vec<Vec3b>> = resized_img.to_vec_2d().unwrap();
        let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();

        interpreter.copy(&vec_1d[..], 0).unwrap();
        // run interpreter
        interpreter.invoke().expect("Invoke [FAILED]");


        // get output
        let output_tensor = interpreter.output(0).unwrap();
        let tensor_data = output_tensor.data::<f32>();

        const LEN_OUTPUT: usize = 48*48*24;
        let mut tensor_bytes = [0u8; LEN_OUTPUT*4];
        client::f32_to_bytes(&tensor_data, &mut tensor_bytes);
        server.send_bytes(&tensor_bytes);

        let mut output_bytes = vec![0u8; LEN_OUTPUT*4];
        let mut output = [0.0f32; LEN_OUTPUT];
        server.receive_bytes(&mut output_bytes); 
        client::bytes_to_f32(&output_bytes, &mut output);

        draw_keypoints(&mut blank, &output, 0.25);
        imshow("MoveNet", &blank).expect("imshow [ERROR]");

        // test lower network locally
        /*
        interpreter2.copy(tensor_data, 0).unwrap();
        interpreter2.invoke().expect("Invoke [FAILED]");
        let output_tensor2 = interpreter2.output(0).unwrap();
        let tensor_data2 = output_tensor2.data::<f32>();
        draw_keypoints(&mut blank, &tensor_data2, 0.25);
        imshow("MoveNet", &blank).expect("imshow [ERROR]");
        */

        dequeue_buffer(&media_fd, &mut qbuffer);

        count += 1;

		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			break;
        }
        // save for debugging
        if debug {
            let mut name: String = String::from("frames_test/");
            name.push_str(&count.to_string());
            save_yuv(name, &image_bytes);
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

