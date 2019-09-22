#![deny(unused_must_use)]
#![recursion_limit = "256"]

mod args_parser;
mod error;
mod filter;
mod json_path;
mod json_value;
mod parse_and_keep;
mod parse_and_throw;
mod parse_basics;
mod parse_query;
mod parse_smart;
mod pipeline;
mod pipeline_builder;
mod unicode_stream;

use combine::parser::Parser;
use std::convert::From;

pub use crate::args_parser::ArgStruct;
use crate::pipeline_builder::PipelineBuilder;

pub fn parse_from_args(args: ArgStruct) -> Result<(), failure::Error> {
    // ParseError<Stream<ReadStream<Stdin>>>
    let pipeline_builder = PipelineBuilder::from(&args);

    let stream = pipeline_builder.build_input_stream()?;

    let mut parser = pipeline_builder.build_parser()?;

    parser
        .easy_parse(stream)
        .map(|(output, _)| output)
        .map_err(|err| err.into())
}
