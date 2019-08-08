use std::convert::From;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;

use crate::filter::Filter;

use crate::args_parser::ArgStruct;
use crate::json_value::JsonValue;

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

    pub fn build_pipeline(&self) -> Result<Box<dyn Pipeline>, String> {
        let output_stage: Box<dyn Pipeline> = if let Some(ref filename) = self.0.output {
            let output_writer = OpenOptions::new()
                .write(true)
                .create(true)
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
}

impl<'a> From<&'a ArgStruct> for PipelineBuilder<'a> {
    fn from(args: &'a ArgStruct) -> PipelineBuilder<'a> {
        PipelineBuilder(&args)
    }
}
