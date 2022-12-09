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

pub fn bytes_to_f32(bytes: &[u8], floats: &mut [f32]) {
    const BYTES_PER_F32: usize = 4;
    let mut i = 0;
    for float_bytes in bytes.chunks(BYTES_PER_F32) {
        let float_bytes: [u8; 4] = float_bytes.try_into().expect("invalid length");
        floats[i] = f32::from_ne_bytes(float_bytes);
        i += 1;
    }
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
    const LEN_UPPER_OUTPUT: usize = 48*48*24;
    loop {
        let mut upper_bytes = [0u8; LEN_UPPER_OUTPUT*4];
        let mut reader = BufReader::new(&stream);

        if let Err(num_bytes) = reader.read_exact(&mut upper_bytes) {
            break;
        }

        let mut upper_floats = [0f32; LEN_UPPER_OUTPUT];
        bytes_to_f32(&upper_bytes, &mut upper_floats);
        interpreter.copy(&upper_floats, 0).unwrap();

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
	let path = format!("../splitter/output/lower.tflite");
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
