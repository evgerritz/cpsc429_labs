use crate::resize_with_padding;
use opencv::core::Vec3b;
use opencv::prelude::*;
use std::net::TcpStream;
use std::str;
use std::io::{self, BufRead, BufReader, Write, Read};

pub struct Server {
    stream: TcpStream,
}

impl Server {
    pub fn new() -> Server {
        Server {
            stream: TcpStream::connect("localhost:8080").expect("could not connect"),
        }
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) {
        println!("sending bytes");
        self.stream.write(bytes).expect("failed to write bytes");
    }

    pub fn receive_bytes(&self, buffer: &mut [u8]) {
        let mut reader = BufReader::new(& self.stream);
        reader.read(buffer).expect("could not read into buffer");
        println!("{}", str::from_utf8(&buffer).expect("invalid utf-8 string"));
    }
}

pub fn image_to_bytes(frame: &Mat) -> Vec<u8> {
    // resize the image as a square, size is 
    let resized_img = resize_with_padding(&frame, [192, 192]);

    // turn Mat into Vec<u8>
    let vec_2d: Vec<Vec<Vec3b>> = resized_img.to_vec_2d().unwrap();
    vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect()
}

pub fn bytes_to_f32(bytes: &[u8], floats: &mut [f32]) {
    const BYTES_PER_F32: usize = 4;
    let mut i = 0;
    for float_bytes in bytes.chunks(BYTES_PER_F32) {
        let float_bytes: [u8; 4] = float_bytes.try_into().expect("invalid length");
        floats[i] = f32::from_ne_bytes(float_bytes);
        i += 1;
    }
}
