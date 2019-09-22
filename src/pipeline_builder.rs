use std::convert::From;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::{stdin, Stdin};

use combine::error::ParseError;
use combine::parser::char::spaces;
use combine::parser::item::eof;
use combine::parser::repeat::skip_many;
use combine::parser::Parser;
use combine::stream::Stream;

use crate::args_parser::ArgStruct;
use crate::error::InitError;
use crate::json_path::JsonPath;
use crate::json_value::NumberVal;
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
    pub fn build_input_stream(&self) -> Result<ReadStream<Stdin>, InitError> {
        Ok(ReadStream::from_read_buffered_normalized(
            stdin(),
            self.0.max_text_length,
        ))
    }

    fn build_output(&self) -> Result<Box<dyn Pipeline>, InitError> {
        if let Some(ref filename) = self.0.output {
            let output_writer = OpenOptions::new()
                .write(true)
                .truncate(!self.0.append)
                .append(self.0.append)
                .create(true)
                .create_new(self.0.force_new)
                .open(filename)
                .map_err(|_: std::io::Error| InitError::UnableToOpenFile {
                    filename: filename.to_string(),
                })?;

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

    pub fn build_parser<I>(&self) -> Result<impl Parser<Input = I, Output = ()>, InitError>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
    {
        let output = self.build_output()?;
        let (filter, pipeline) = parse_query(self.0.max_text_length, output, &self.0.query)?;
        let state = ParserState::new(pipeline, filter);
        let state_finisher = state.clone();

        Ok(spaces()
            .with(skip_many(json_smart(state, self.0.max_text_length)).skip(eof()))
            .map(move |_| {
                state_finisher.finish().unwrap();
                ()
            }))
    }
}

impl<'a> From<&'a ArgStruct> for PipelineBuilder<'a> {
    fn from(args: &'a ArgStruct) -> PipelineBuilder<'a> {
        PipelineBuilder(&args)
    }
}
