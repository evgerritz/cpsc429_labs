use std::net::TcpStream;
use std::str;
use std::io::{self, BufRead, BufReader, Write, Read};

pub struct Server {
    stream: TcpStream,
}

impl Server {
    pub fn new() -> Server {
        Server {
            stream: TcpStream::connect("127.0.0.1:8888").expect("could not connect"),
        }
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) {
        self.stream.write(bytes).expect("failed to write bytes");
    }

    pub fn receive_bytes(&self, buffer: &mut [u8]) {
        let mut reader = BufReader::new(& self.stream);
        reader.read(buffer).expect("could not read into buffer");
        println!("{}", str::from_utf8(&buffer).expect("invalid utf-8 string"));
    }
}
