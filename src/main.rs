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

use std::io::stdin;

use combine::parser::Parser;
use structopt::StructOpt;

use crate::parse_smart::{json_smart, ParserState};
use crate::pipeline::PipelineBuilder;
use crate::unicode_stream::ReadStream;

use crate::args_parser::ArgStruct;

fn main() {
    let _ = include_str!("../Cargo.toml"); //Trigger the rebuild automatism when Cargo.toml is changed
    let args = ArgStruct::from_args();

    let stream = ReadStream::from_read_buffered(stdin());

    let pipeline_builder = PipelineBuilder::from(&args);

    let filter = pipeline_builder.build_filter().unwrap();
    let pipeline = pipeline_builder.build_pipeline().unwrap();
    let state = ParserState::new(pipeline, filter);

    // TODO: Parse stream of objects (using many::<(), _> or else)
    json_smart(state).easy_parse(stream).unwrap();
}
