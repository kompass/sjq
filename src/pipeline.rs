use std::convert::From;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;
use std::io::{stdin, Stdin};

use combine::stream::Stream;
use combine::error::ParseError;
use combine::parser::Parser;
use combine::parser::repeat::skip_many;
use combine::parser::item::eof;

use crate::filter::Filter;
use crate::args_parser::ArgStruct;
use crate::json_value::JsonValue;
use crate::unicode_stream::ReadStream;
use crate::parse_smart::{json_smart, ParserState};

pub trait Pipeline {
    fn ingest(&mut self, item: JsonValue) -> Result<(), ()>;
}

pub struct WriteStage<W: Write>(W);

impl<W: Write> Pipeline for WriteStage<W> {
    fn ingest(&mut self, item: JsonValue) -> Result<(), ()> {
        serde_json::to_writer(&mut self.0, &item).unwrap();
        writeln!(&mut self.0).unwrap();
        Ok(())
    }
}

pub struct WritePrettyStage<W: Write>(W);

impl<W: Write> Pipeline for WritePrettyStage<W> {
    fn ingest(&mut self, item: JsonValue) -> Result<(), ()> {
        serde_json::to_writer_pretty(&mut self.0, &item).unwrap();
        writeln!(&mut self.0).unwrap();
        Ok(())
    }
}

pub struct AddFieldStage {
    key: String,
    value: JsonValue,
    output: Box<dyn Pipeline>,
}

impl AddFieldStage {
    pub fn new(output: Box<dyn Pipeline>, key: &str, value: JsonValue) -> AddFieldStage {
        AddFieldStage {
            key: key.to_string(),
            value,
            output,
        }
    }
}

impl Pipeline for AddFieldStage {
    fn ingest(&mut self, mut item: JsonValue) -> Result<(), ()> {
        if let JsonValue::Object(ref mut obj) = item {
            obj.insert(self.key.clone(), self.value.clone());

            self.output.ingest(item)
        } else {
            Err(())
        }
    }
}

pub struct PipelineBuilder<'a>(&'a ArgStruct);

impl<'a> PipelineBuilder<'a> {
    pub fn build_filter(&self) -> Result<Filter, String> {
        Filter::from_str(&self.0.query)
    }

    pub fn build_input_stream(&self) -> Result<ReadStream<Stdin>, String> {
        Ok(ReadStream::from_read_buffered(stdin(), self.0.max_text_length))
    }

    pub fn build_pipeline(&self) -> Result<Box<dyn Pipeline>, String> {
        let output_stage: Box<dyn Pipeline> = if let Some(ref filename) = self.0.output {
            let output_writer = OpenOptions::new()
                .write(true)
                .append(self.0.append)
                .create_new(self.0.force_new)
                .open(filename)
                .unwrap();

            if self.0.pretty {
                Box::new(WritePrettyStage(output_writer))
            } else {
                Box::new(WriteStage(output_writer))
            }
        } else {
            let output_writer = stdout();

            if self.0.pretty {
                Box::new(WritePrettyStage(output_writer))
            } else {
                Box::new(WriteStage(output_writer))
            }
        };

        Ok(Box::new(AddFieldStage::new(
            output_stage,
            "pipeline_status",
            JsonValue::String("running".to_string()),
        )))
    }

    pub fn build_parser<I>(&self) -> Result<impl Parser<Input = I, Output = ()>, String>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
    {
        let filter = self.build_filter().unwrap();
        let pipeline = self.build_pipeline().unwrap();
        let state = ParserState::new(pipeline, filter);

        // TODO: Parse stream of objects (using many::<(), _> or else)
        Ok(skip_many(json_smart(state, self.0.max_text_length)).skip(eof()))
    }
}

impl<'a> From<&'a ArgStruct> for PipelineBuilder<'a> {
    fn from(args: &'a ArgStruct) -> PipelineBuilder<'a> {
        PipelineBuilder(&args)
    }
}
