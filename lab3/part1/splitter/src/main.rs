extern crate flatbuffers;
 
#[allow(dead_code, unused_imports)]
mod schema_generated;
//use schema_generated;

use std::fs::File;
use std::io::Read;
use std::io::Write;

const SPLIT_OP_OUTPUT: i32 = 255;
const FILE_IDENTIFIER: &str = "TFL3";

fn main() {
    let mut input_f = File::open("input/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite").unwrap();
    let mut buf = Vec::new();
    input_f.read_to_end(&mut buf).unwrap();
    let mut upper = flatbuffers::root::<schema_generated::tflite::Model>(&buf).expect("invalid model").unpack();  
    //let lower = flatbuffers::root::<schema_generated::tflite::Model>(&buf).expect("invalid model").unpack();  

    let mut split_index = 0;
    for operator in upper.subgraphs.as_ref().unwrap()[0].operators.as_ref().unwrap() {
        if operator.outputs.as_ref().unwrap()[0] == 255 {
            println!("{:?}", operator);
            break;
        }
        split_index += 1;
    }

    let subgraphs = upper.subgraphs.as_mut().unwrap();
    let ops_p = subgraphs[0].operators.as_mut().unwrap();
    *ops_p = (ops_p[..split_index]).to_vec();

    /*for operator in upper.subgraphs.as_ref().unwrap()[0].operators.as_ref().unwrap() {
        println!("{:?}", operator);
    }*/

    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let offset = upper.pack(&mut builder);
    builder.finish(offset, Some(FILE_IDENTIFIER));
    let upper_bytes = builder.finished_data();
    
    let mut upper_f = File::create("output/upper.tflite").unwrap();
    //let mut lower_f = File::open("output/lower.tflite").unwrap();
    upper_f.write(upper_bytes).unwrap();
}
