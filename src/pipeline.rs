use std::convert::From;
use std::ops::Deref;
use std::str::FromStr;

use crate::filter::Filter;

use crate::args_parser::ArgStruct;
use crate::json_value::JsonValue;

pub trait Pipeline {
    fn ingest(&self, item: JsonValue) -> Result<(), ()>;
}

pub struct StdoutStage();

impl Pipeline for StdoutStage {
    fn ingest(&self, item: JsonValue) -> Result<(), ()> {
        println!("{}", item);
        Ok(())
    }
}

pub struct AddFieldStage<O: Pipeline> {
    key: String,
    value: JsonValue,
    output: O,
}

impl<O: Pipeline> AddFieldStage<O> {
    pub fn new(output: O, key: &str, value: JsonValue) -> AddFieldStage<O> {
        AddFieldStage {
            key: key.to_string(),
            value,
            output,
        }
    }
}

impl<O: Pipeline> Pipeline for AddFieldStage<O> {
    fn ingest(&self, mut item: JsonValue) -> Result<(), ()> {
        if let JsonValue::Object(ref mut obj) = item {
            obj.insert(self.key.clone(), self.value.clone());

            self.output.ingest(item)
        } else {
            Err(())
        }
    }
}

pub struct PipelineBuilder<'a> {
    query: &'a str,
    pretty: bool,
    output_filename: Option<&'a str>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn build_filter(&self) -> Result<Filter, String> {
        Filter::from_str(self.query)
    }
}

impl<'a> From<&'a ArgStruct> for PipelineBuilder<'a> {
    fn from(args: &'a ArgStruct) -> PipelineBuilder<'a> {
        PipelineBuilder {
            query: &args.query,
            pretty: args.pretty,
            output_filename: args.output.as_ref().map(|t| t.deref()), //TODO: Use Option::deref when it will be stable
        }
    }
}
