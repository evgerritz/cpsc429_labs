use opencv::core::{flip, Vec3b};
use opencv::videoio::*;
use opencv::{
	prelude::*,
	videoio,
	highgui::*,
};

mod utils;
use utils::*;
use tflitec::interpreter::{Interpreter, Options};

mod client;
use client::Server;

/*fn main() {
    let mut server = Server::new();
    let input: Vec<u8> = vec![1,2,3,4];
    let mut output: Vec<u8> = vec![0,0,0,0];

    server.send_bytes(&input[..]);
    server.receive_bytes(&mut output[..]); 
    println!("{:?}", output);
}*/

fn main() {
	// load model and create interpreter
	let options = Options::default();
	let path = format!("resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite");
	let interpreter = Interpreter::with_model_path(&path, Some(options)).unwrap();
	interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");
	// Resize input
	
	// open camera
	let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap(); // 0 is the default camera
	videoio::VideoCapture::is_opened(&cam).expect("Open camera [FAILED]");
	cam.set(CAP_PROP_FPS, 30.0).expect("Set camera FPS [FAILED]");

    // establish connection to server
    let mut server = Server::new();
    const LEN_OUTPUT: usize = 17*3; 

	loop {
		let mut frame = Mat::default();
		cam.read(&mut frame).expect("VideoCapture: read [FAILED]");

		if frame.size().unwrap().width > 0 {
            // flip the image horizontally
            let mut flipped = Mat::default();
            flip(&frame, &mut flipped, 1).expect("flip [FAILED]");

            let image_bytes: Vec<u8> = client::image_to_bytes(&flipped);
            let mut output_bytes: [u8; LEN_OUTPUT*4] = [0; LEN_OUTPUT*4];
            let mut output: [f32; LEN_OUTPUT] = [0.0; LEN_OUTPUT];

            server.send_bytes(&image_bytes);
            server.receive_bytes(&mut output_bytes); 

            client::bytes_to_f32(&output_bytes, &mut output);
			draw_keypoints(&mut flipped, &output, 0.25);
			imshow("MoveNet", &flipped).expect("imshow [ERROR]");
		}
		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			break;
		}
	}
}

