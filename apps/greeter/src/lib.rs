extern crate zenith;
extern crate zenith_derive;

use serde::{Deserialize, Serialize};
use std::slice;
use zenith::prelude::*;
use zenith_derive::main;
// Initialize module memory

/*
    Create attribute zenith::run(ZenDeserializer, usize, usize)

    This takes the following arguments:
        input_buffer_size
        output_buffer_size
        deserializer -> Enum
            JSON: Interpret bytes as JSON string
            BinCode: Interpret bytes as bincode serialized struct
            default None: No serialization

    The attribute should:
        Generate the input and output buffers based on the size arguments
        Generate memory access utilities that are exported in the WASI module

    Example program:


        struct MyOutput {
            greeting: String
        }

        #[zenith::run(JSON, 1024, 1024)]
        pub fn run(input: MyStruct) -> i32 {
            let output = MyOutput { greeting: format!("Hello, {}", input.name) }
            let bytes = bincode.serialize(&output).as_slice();
            write_output(bytes)
        }
*/
#[derive(Deserialize, Serialize)]
pub struct MyInput {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct MyOutput {
    pub greeting: String,
}

#[main(1024, 1024, zenith::prelude::ZenDeserializer::Bincode)]
pub fn run(input: MyInput) -> MyOutput {
    let greeting = format!("Hello, {}", input.name);
    let output = MyOutput { greeting };
    output
}
