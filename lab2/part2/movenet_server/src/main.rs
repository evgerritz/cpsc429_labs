use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::io::Write;
use std::thread;


fn handle_client(mut stream: TcpStream) {
    const IM_SIZE: usize = 384 * 288;
    let mut image_bytes: [u8; IM_SIZE];

    let stream: Vec<u8> = stream.read(&mut image_bytes).expect("couldn't read from stream");

    interpreter.copy(&image_bytes[..], 0).unwrap();

    // run interpreter
    interpreter.invoke().expect("Invoke [FAILED]");

    // get output
    let output_tensor = interpreter.output(0).unwrap();
    let tensor_data = output_tensor.data::<f32>()

    stream.write(&tensor_data[..]).unwrap();
}

pub fn main() {
    let listener: TcpListener = TcpListener::bind("0.0.0.0:8888").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(_) => {
                println!("Error");
            }
        } 
    }
}
