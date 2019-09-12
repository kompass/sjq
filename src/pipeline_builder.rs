use std::convert::From;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::{stdin, Stdin};

use combine::error::ParseError;
use combine::parser::item::eof;
use combine::parser::repeat::skip_many;
use combine::parser::Parser;
use combine::stream::Stream;

use crate::args_parser::ArgStruct;
use crate::json_path::JsonPath;
use crate::parse_basics::NumberVal;
use crate::parse_query::parse_query;
use crate::parse_smart::{json_smart, ParserState};
use crate::pipeline::Pipeline;
use crate::pipeline::*;
use crate::unicode_stream::ReadStream;

pub struct PipelineBuilder<'a>(&'a ArgStruct);

pub enum StageArg {
    Number(NumberVal),
    String(String),
    Path(JsonPath),
}

impl<'a> PipelineBuilder<'a> {
    pub fn build_input_stream(&self) -> Result<ReadStream<Stdin>, String> {
        Ok(ReadStream::from_read_buffered_normalized(
            stdin(),
            self.0.max_text_length,
        ))
    }

    fn build_output(&self) -> Result<Box<dyn Pipeline>, String> {
        if let Some(ref filename) = self.0.output {
            let output_writer = OpenOptions::new()
                .write(true)
                .append(self.0.append)
                .create_new(self.0.force_new)
                .open(filename)
                .unwrap();

            if self.0.pretty {
                Ok(Box::new(WritePrettyStage::new(output_writer)))
            } else {
                Ok(Box::new(WriteStage::new(output_writer)))
            }
        } else {
            let output_writer = stdout();

            if self.0.pretty {
                Ok(Box::new(WritePrettyStage::new(output_writer)))
            } else {
                Ok(Box::new(WriteStage::new(output_writer)))
            }
        }
    }

    pub fn build_parser<I>(&self) -> Result<impl Parser<Input = I, Output = ()>, String>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
    {
        let output = self.build_output().unwrap();
        let (filter, pipeline) =
            parse_query(self.0.max_text_length, output, &self.0.query).unwrap();
        let state = ParserState::new(pipeline, filter);
        let state_finisher = state.clone();

        Ok(skip_many(json_smart(state, self.0.max_text_length)).skip(eof()).map(move |_| { state_finisher.finish().unwrap(); () }))
    }
}

impl<'a> From<&'a ArgStruct> for PipelineBuilder<'a> {
    fn from(args: &'a ArgStruct) -> PipelineBuilder<'a> {
        PipelineBuilder(&args)
    }
}
