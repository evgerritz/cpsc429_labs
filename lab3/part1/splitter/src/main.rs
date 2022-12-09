extern crate flatbuffers;
 
#[allow(dead_code, unused_imports)]
mod schema_generated;
use schema_generated::tflite::*;

use std::fs::File;
use std::io::Read;
use std::io::Write;

const SPLIT_OP_OUTPUT: i32 = 255;
const FILE_IDENTIFIER: &str = "TFL3";

fn main() {
    let mut input_f = File::open("input/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite").unwrap();
    let mut buf = Vec::new();
    input_f.read_to_end(&mut buf).unwrap();
    let mut upper = flatbuffers::root::<Model>(&buf).expect("invalid model").unpack();  
    let mut lower = flatbuffers::root::<Model>(&buf).expect("invalid model").unpack();  

    // split operators
    let mut split_index = 0;
    for operator in upper.subgraphs.as_ref().unwrap()[0].operators.as_ref().unwrap() {
        split_index += 1;
        if operator.outputs.as_ref().unwrap()[0] == SPLIT_OP_OUTPUT {
            break;
        }
    }

    
    // Print all operators, tensors, buffers in original network for debugging
    /* 
    let mut i = 0;
    for operator in upper_subgraphs[0].operators.as_ref().unwrap() {
        println!("Operator: {:?}, {:?}", i, operator);
        i += 1;
    }
    i=0;
    for tensor in upper_subgraphs[0].tensors.as_ref().unwrap() {
        println!("Tensor: {:?}, {:?}", i, tensor);
        i += 1;
    }
    i = 0;
    for buffer in upper.buffers.as_ref().unwrap(){
        println!("Buffer: {:?}, {:?}", i, buffer);
        i += 1;
    }
    */

    
    // split the operators
    let upper_subgraphs = upper.subgraphs.as_mut().unwrap();
    let lower_subgraphs = lower.subgraphs.as_mut().unwrap();
    *lower_subgraphs[0].operators.as_mut().unwrap() = upper_subgraphs[0].operators.as_mut().unwrap()
        .split_off(split_index);

    // add the dequant/quant operators
    let dequant_out_buffer = BufferT::default(); // {data: Some(vec![0; 48*48*24*4])}; - this
                                                 // results in a segfault, so we use an empty buffer
                                                 // instead
    upper.buffers.as_mut().unwrap().push(dequant_out_buffer);
    let quant_in_buffer = BufferT::default(); //{data: Some(vec![0; 48*48*24*4])};
    lower.buffers.as_mut().unwrap().push(quant_in_buffer);

    let mut dequant_out_tensor = TensorT::default();
    dequant_out_tensor = TensorT {
        shape: Some(vec![1,48,48,24]),
        buffer: 335,
        name: Some("StatefulPartitionedCall:0".to_string()),
        ..dequant_out_tensor
    };

    let mut quant_in_tensor = TensorT::default();
    quant_in_tensor = TensorT {
        shape: Some(vec![1,48,48,24]),
        buffer: 335,
        name: Some("Cast".to_string()),
        ..quant_in_tensor
    };

    upper_subgraphs[0].tensors.as_mut().unwrap().push(dequant_out_tensor);
    lower_subgraphs[0].tensors.as_mut().unwrap().push(quant_in_tensor);

    let mut dequant_op = OperatorT::default();
    dequant_op = OperatorT {
        opcode_index: 13,
        inputs: Some(vec![255]),
        outputs: Some(vec![333]),
        ..dequant_op
    };

    let mut quant_op = OperatorT::default();
    quant_op = OperatorT {
        opcode_index: 1,
        inputs: Some(vec![333]),
        outputs: Some(vec![255]),
        ..quant_op
    };

    upper_subgraphs[0].operators.as_mut().unwrap().push(dequant_op);
    lower_subgraphs[0].operators.as_mut().unwrap().push(quant_op);


    // change inputs/outputs
    *upper_subgraphs[0].outputs.as_mut().unwrap() = vec![333];
    *lower_subgraphs[0].inputs.as_mut().unwrap() = vec![333];


    // done editing the split network, save to disk
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
