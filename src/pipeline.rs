use std::cell::Cell;
use std::convert::From;
use std::convert::TryFrom;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::Write;
use std::io::{stdin, Stdin};
use std::str::FromStr;

use combine::error::ParseError;
use combine::parser::item::eof;
use combine::parser::repeat::skip_many;
use combine::parser::Parser;
use combine::stream::Stream;

use crate::args_parser::ArgStruct;
use crate::filter::Filter;
use crate::json_path::JsonPath;
use crate::json_value::JsonValue;
use crate::parse_basics::NumberVal;
use crate::parse_smart::{json_smart, ParserState};
use crate::unicode_stream::ReadStream;

pub trait Pipeline {
    /// Ingest stream items one by one, in the right order.
    fn ingest(&mut self, item: JsonValue) -> Result<(), String>;

    /// Do the necessary when the stream is done.
    /// After this, the Pipeline returns in default mode,
    /// like if it ingested nothing.
    /// It has to call the finish method of its output(s).
    /// The main use of this method is for aggregating stages.
    fn finish(&mut self) -> Result<(), String>;
}

pub struct WriteStage<W: Write>(W);

impl<W: Write> Pipeline for WriteStage<W> {
    fn ingest(&mut self, item: JsonValue) -> Result<(), String> {
        serde_json::to_writer(&mut self.0, &item).unwrap();
        writeln!(&mut self.0).unwrap();
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        Ok(())
    }
}

pub struct WritePrettyStage<W: Write>(W);

impl<W: Write> Pipeline for WritePrettyStage<W> {
    fn ingest(&mut self, item: JsonValue) -> Result<(), String> {
        serde_json::to_writer_pretty(&mut self.0, &item).unwrap();
        writeln!(&mut self.0).unwrap();
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
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
    fn ingest(&mut self, mut item: JsonValue) -> Result<(), String> {
        if let JsonValue::Object(ref mut obj) = item {
            obj.insert(self.key.clone(), self.value.clone());

            self.output.ingest(item)
        } else {
            Err("Can't add a field to a non-object value.".to_string())
        }
    }

    fn finish(&mut self) -> Result<(), String> {
        self.output.finish()
    }
}

pub struct SumStage {
    acc: Cell<Option<NumberVal>>,
    summed_value: JsonPath,
    strict: bool,
    output: Box<dyn Pipeline>,
}

impl SumStage {
    pub fn new(output: Box<dyn Pipeline>, summed_value: JsonPath, strict: bool) -> SumStage {
        SumStage {
            acc: Cell::new(None),
            summed_value,
            strict,
            output,
        }
    }
}

impl Pipeline for SumStage {
    fn ingest(&mut self, item: JsonValue) -> Result<(), String> {
        if let Some(item) = item.select(&self.summed_value) {
            match (self.acc.get(), item) {
                (None, item) => self.acc.set(Some(NumberVal::try_from(item).unwrap())),
                (Some(NumberVal::Integer(acc)), &JsonValue::Integer(i)) => {
                    self.acc.set(Some(NumberVal::Integer(acc + i)))
                }
                (Some(NumberVal::Float(acc)), &JsonValue::Float(f)) => {
                    self.acc.set(Some(NumberVal::Float(acc + f)))
                }
                (Some(NumberVal::Integer(acc)), &JsonValue::Float(f)) => {
                    self.acc.set(Some(NumberVal::Float((acc as f64) + f)))
                }
                (Some(NumberVal::Float(acc)), &JsonValue::Integer(i)) => {
                    self.acc.set(Some(NumberVal::Float(acc + (i as f64))))
                }
                _ => return Err("Impossible to sum a non-number value.".to_string()),
            }

            Ok(())
        } else if self.strict {
            Err("Missing summed value.".to_string())
        } else {
            Ok(())
        }
    }

    fn finish(&mut self) -> Result<(), String> {
        self.output
            .ingest(self.acc.get().unwrap_or(NumberVal::Integer(0)).into())
            .unwrap();
        self.output.finish().unwrap();
        self.acc.replace(None);

        Ok(())
    }
}

pub struct PipelineBuilder<'a>(&'a ArgStruct);

impl<'a> PipelineBuilder<'a> {
    pub fn build_filter(&self) -> Result<Filter, String> {
        Filter::from_str(&self.0.query)
    }

    pub fn build_input_stream(&self) -> Result<ReadStream<Stdin>, String> {
        Ok(ReadStream::from_read_buffered_normalized(
            stdin(),
            self.0.max_text_length,
        ))
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

        Ok(skip_many(json_smart(state, self.0.max_text_length)).skip(eof()))
    }
}

impl<'a> From<&'a ArgStruct> for PipelineBuilder<'a> {
    fn from(args: &'a ArgStruct) -> PipelineBuilder<'a> {
        PipelineBuilder(&args)
    }
}
