use opencv::core::flip;
use opencv::videoio::*;
use opencv::{
	prelude::*,
	videoio,
	highgui::*,
};

mod utils;
use utils::*;

mod client;
use client::Server;

use std::time::Instant;
use std::time::Duration;


fn main() {
	// open camera
	let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(); // 0 is the default camera
	videoio::VideoCapture::is_opened(&cam).expect("Open camera [FAILED]");
	cam.set(CAP_PROP_FPS, 30.0).expect("Set camera FPS [FAILED]");

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
}

