use std::net::{TcpListener, TcpStream};
use tflitec::interpreter::{Interpreter, Options};
use std::io::Read;
use std::io::Write;
use std::io::BufReader;

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
    loop {
        const IM_SIZE: usize = 192*192*3;
        let mut image_bytes: [u8; IM_SIZE] = [0; IM_SIZE];
        let mut reader = BufReader::new(&stream);

        if let Err(num_bytes) = reader.read_exact(&mut image_bytes) {
            break;
        }

        interpreter.copy(&image_bytes, 0).unwrap();

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
