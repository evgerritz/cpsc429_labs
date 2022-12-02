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
    let mut lower = flatbuffers::root::<schema_generated::tflite::Model>(&buf).expect("invalid model").unpack();  

    let mut split_index = 0;
    for operator in upper.subgraphs.as_ref().unwrap()[0].operators.as_ref().unwrap() {
        split_index += 1;
        if operator.outputs.as_ref().unwrap()[0] == SPLIT_OP_OUTPUT {
            break;
        }
    }

    let upper_subgraphs = upper.subgraphs.as_mut().unwrap();
    let upper_ops_p = upper_subgraphs[0].operators.as_mut().unwrap();
    *upper_ops_p = (upper_ops_p[..split_index]).to_vec();

    let lower_subgraphs = lower.subgraphs.as_mut().unwrap();
    let lower_ops_p = lower_subgraphs[0].operators.as_mut().unwrap();
    *lower_ops_p = (lower_ops_p[split_index..]).to_vec();

    let mut upper_builder = flatbuffers::FlatBufferBuilder::new();
    let upper_offset = upper.pack(&mut upper_builder);
    upper_builder.finish(upper_offset, Some(FILE_IDENTIFIER));
    let upper_bytes = upper_builder.finished_data();

    let mut lower_builder = flatbuffers::FlatBufferBuilder::new();
    let lower_offset = lower.pack(&mut lower_builder);
    lower_builder.finish(lower_offset, Some(FILE_IDENTIFIER));
    let lower_bytes = lower_builder.finished_data();
    
    let mut upper_f = File::create("output/upper.tflite").unwrap();
    upper_f.write(upper_bytes).unwrap();
    
    let mut lower_f = File::create("output/lower.tflite").unwrap();
    lower_f.write(lower_bytes).unwrap();

}
