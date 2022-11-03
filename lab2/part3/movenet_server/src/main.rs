use std::net::{TcpListener, TcpStream};
use tflitec::interpreter::{Interpreter, Options};
use std::io::Read;
use std::io::Write;
use std::io::BufReader;
use opencv::{
    prelude::*,
    imgproc::*,
    core::*,
};
use libc;

fn YUV_2_B(y: i32, u: i32) -> i32 {
    let y = y as f32;
    let u = u as f32;
    (y + 1.732446 * (u - 128.0)) as i32
}
fn YUV_2_G(y: i32, u: i32, v: i32) -> i32 {
    let y = y as f32;
    let u = u as f32;
    let v = v as f32;
    (y - 0.698001 * (u - 128.0) - 0.703125 * (v - 128.0)) as i32
}
fn YUV_2_R(y: i32, v: i32) -> i32 {
    let y = y as f32;
    let v = v as f32;
    (y + 1.370705 * (v - 128.0)) as i32
}

// adapted from https://github.com/kd40629rtlrtl/yuv422_to_rgb/blob/master/yuv_to_rgb.c
pub fn yuv422_to_rgb24(in_buf: &[u8], out_buf: &mut [u8], width: i32, height: i32) {
    let len: i32 = width * height;
    let yData: &[u8] = in_buf;
    let vData: &[u8] = in_buf; 
    let uData: &[u8] = in_buf;

    let mut bgr: [i32; 3] = [0; 3];
    let mut yIdx: usize;
    let mut uIdx: usize;
    let mut vIdx: usize;
    let mut idx: usize;
    for y in 0..height {
        for x in 0..width {
           	yIdx = 2*((y*width) + x) as usize;
            uIdx = (4*(((y*width) + x)>>1) + 1) as usize;
            vIdx = (4*(((y*width) + x)>>1) + 3) as usize;

            if yIdx >= in_buf.len() || uIdx >= in_buf.len() || vIdx >= in_buf.len() {
                return; 
            }
            bgr[0] = YUV_2_B(yData[yIdx].into(), uData[uIdx].into()); 
			bgr[1] = YUV_2_G(yData[yIdx].into(), uData[uIdx].into(), vData[vIdx].into());
			bgr[2] = YUV_2_R(yData[yIdx].into(), vData[vIdx].into()); 

			for k in 0..3 as usize {
                idx = ((y * width + x) * 3 + (k as i32)) as usize;
                if bgr[k] >= 0 && bgr[k] <= 255 {
                    out_buf[idx] = bgr[k] as u8;
                } else if bgr[k] < 0 {
                    out_buf[idx] = 0;
                } else {
                    out_buf[idx] = 255;
                }
            }
        }
    }
}

pub fn resize_with_padding(img: &Mat, new_shape: [i32;2]) -> Mat {
	let img_shape = [img.cols(), img.rows()];
	let width: i32;
	let height: i32;
	if img_shape[0] as f64 / img_shape[1] as f64 > new_shape[0] as f64 / new_shape[1] as f64 {
		width = new_shape[0];
		height = (new_shape[0] as f64 / img_shape[0] as f64 * img_shape[1] as f64) as i32;
	} else {
		width = (new_shape[1] as f64 / img_shape[1] as f64 * img_shape[0] as f64) as i32;
		height = new_shape[1];
	}

	let mut resized = Mat::default();
	resize(
		img,
		&mut resized,
		Size { width, height },
		0.0, 0.0,
		INTER_LINEAR)
		.expect("resize_with_padding: resize [FAILED]");

	let delta_w = new_shape[0] - width;
	let delta_h = new_shape[1] - height;
	let (top, bottom) = (delta_h / 2, delta_h - delta_h / 2);
	let (left, right) = (delta_w / 2, delta_w - delta_w / 2);
		
	let mut rslt = Mat::default();
	copy_make_border(
		&resized,
		&mut rslt,
		top, bottom, left, right,
		BORDER_CONSTANT,
		Scalar::new(0.0, 0.0, 0.0, 0.0))
		.expect("resize_with_padding: copy_make_border [FAILED]");
	rslt
}

fn f32_to_bytes(floats: &[f32], bytes: &mut [u8]) {
    let mut i = 0;
    for float in floats {
        for byte in float.to_ne_bytes() {
            bytes[i] = byte;
            i += 1;
        }
    }
}


fn handle_client(mut stream: TcpStream, interpreter: &Interpreter) {
    stream.set_read_timeout(None).expect("set read timeout failed");
    stream.set_write_timeout(None).expect("set write timeout failed");
    const IMG_WIDTH: i32 = 320;
    const IMG_HEIGHT: i32 = 180;
    const IM_SIZE: usize = 118784;
    loop {
        let mut image_bytes: [u8; IM_SIZE] = [0; IM_SIZE];
        let mut reader = BufReader::new(&stream);

        if let Err(num_bytes) = reader.read_exact(&mut image_bytes) {
            break;
        }


        let mut rgb_bytes = vec![0; IM_SIZE*3/2];
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

        const LEN_OUTPUT: usize = 17*3;
        let mut tensor_bytes: [u8; LEN_OUTPUT*4] = [0; LEN_OUTPUT*4];
        f32_to_bytes(&tensor_data, &mut tensor_bytes);

        stream.write(&tensor_bytes).unwrap();
    }
}

pub fn main() {
	let options = Options::default();
	let path = format!("resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite");
	let interpreter = Interpreter::with_model_path(&path, Some(options)).unwrap();
	interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");

    //let listener: TcpListener = TcpListener::bind("0.0.0.0:8080").unwrap();
    // use this for zoo:
    let listener: TcpListener = TcpListener::bind("10.0.2.15:8888").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, &interpreter);
            }
            Err(_) => {
                println!("Error");
            }
        } 
    }
}
