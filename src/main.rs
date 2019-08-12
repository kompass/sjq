#![deny(unused_must_use)]

mod args_parser;
mod filter;
mod json_path;
mod json_value;
mod parse_and_keep;
mod parse_and_throw;
mod parse_basics;
mod parse_smart;
mod pipeline;
mod unicode_stream;

use combine::parser::Parser;
use structopt::StructOpt;

use crate::pipeline::PipelineBuilder;

use crate::args_parser::ArgStruct;

// TODO: Remove as unwrap as possible in all src/ by panics and error messages (fail fast)

fn main() {
    let _ = include_str!("../Cargo.toml"); //Trigger the rebuild automatism when Cargo.toml is changed
    let args = ArgStruct::from_args();

    let pipeline_builder = PipelineBuilder::from(&args);

    let stream = pipeline_builder.build_input_stream().unwrap();

    let mut parser = pipeline_builder.build_parser().unwrap();

    parser.easy_parse(stream).unwrap();
}
