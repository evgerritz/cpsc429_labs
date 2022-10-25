use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::io::Write;
use std::thread;


fn handle_client(mut stream: TcpStream) {
    let mut read: [u8;4] = [0,0,0,0];
    match stream.read(&mut read) {
        Ok(n) => {
            println!("{:?}", read);
            for i in 0..read.len() {
                read[i] += 1; 
            }
            println!("{:?}", read);
            stream.write(&read[..]).unwrap();
        }
        Err(err) => {
            println!("{err:?}");
        }
    }
}

pub fn main() {
    let listener: TcpListener = TcpListener::bind("10.0.2.15:8888").unwrap();
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
