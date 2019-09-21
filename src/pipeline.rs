use std::cell::Cell;
use std::io::Write;

use crate::error::PipelineError;
use crate::json_path::JsonPath;
use crate::json_value::{JsonValue, NumberVal};
use crate::pipeline_builder::StageArg;

pub trait Pipeline {
    /// Ingest stream items one by one, in the right order.
    fn ingest(&mut self, item: JsonValue) -> Result<(), PipelineError>;

    /// Do the necessary when the stream is done.
    /// After this, the Pipeline returns in default mode,
    /// like if it ingested nothing.
    /// It has to call the finish method of its output(s).
    /// The main use of this method is for aggregating stages.
    fn finish(&mut self) -> Result<(), PipelineError>;
}

pub struct WriteStage<W: Write>(W);

impl<W: Write> WriteStage<W> {
    pub fn new(output: W) -> WriteStage<W> {
        WriteStage(output)
    }
}

impl<W: Write> Pipeline for WriteStage<W> {
    fn ingest(&mut self, item: JsonValue) -> Result<(), PipelineError> {
        serde_json::to_writer(&mut self.0, &item).unwrap();
        writeln!(&mut self.0).unwrap();
        Ok(())
    }

    fn finish(&mut self) -> Result<(), PipelineError> {
        Ok(())
    }
}

pub struct WritePrettyStage<W: Write>(W);

impl<W: Write> WritePrettyStage<W> {
    pub fn new(output: W) -> WritePrettyStage<W> {
        WritePrettyStage(output)
    }
}

impl<W: Write> Pipeline for WritePrettyStage<W> {
    fn ingest(&mut self, item: JsonValue) -> Result<(), PipelineError> {
        serde_json::to_writer_pretty(&mut self.0, &item).unwrap();
        writeln!(&mut self.0).unwrap();
        Ok(())
    }

    fn finish(&mut self) -> Result<(), PipelineError> {
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

    pub fn from_args(
        output: Box<dyn Pipeline>,
        args: &[StageArg],
    ) -> Result<Box<dyn Pipeline>, String> {
        if args.len() != 2 {
            Err("add_field : Wrong number of arguments.".to_string())
        } else {
            if let (StageArg::String(ref key), StageArg::String(ref value)) =
                (args.get(0).unwrap(), args.get(1).unwrap())
            {
                Ok(Box::new(Self::new(
                    output,
                    &key,
                    JsonValue::normalized_string(&value),
                )))
            } else {
                Err("add_field : Wrong type of arguments.".to_string())
            }
        }
    }
}

impl Pipeline for AddFieldStage {
    fn ingest(&mut self, mut item: JsonValue) -> Result<(), PipelineError> {
        if let JsonValue::Object(ref mut obj) = item {
            obj.insert(self.key.clone(), self.value.clone());

            self.output.ingest(item)
        } else {
            Err(PipelineError::NotAnObject{value: item})
        }
    }

    fn finish(&mut self) -> Result<(), PipelineError> {
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

    pub fn from_args(
        output: Box<dyn Pipeline>,
        args: &[StageArg],
    ) -> Result<Box<dyn Pipeline>, String> {
        if args.len() != 1 {
            Err("sum : Wrong number of arguments.".to_string())
        } else {
            if let StageArg::Path(ref path) = args.get(0).unwrap() {
                Ok(Box::new(Self::new(output, path.clone(), false)))
            } else {
                Err("sum : Wrong type of arguments.".to_string())
            }
        }
    }
}

impl Pipeline for SumStage {
    fn ingest(&mut self, item: JsonValue) -> Result<(), PipelineError> {
        if let Some(item) = item.select(&self.summed_value) {
            if let JsonValue::Number(item_val) = item {
                match (self.acc.get(), item_val) {
                    (None, item_val) => self.acc.set(Some(*item_val)),
                    (Some(NumberVal::Integer(acc)), &NumberVal::Integer(i)) => {
                        self.acc.set(Some(NumberVal::Integer(acc + i)))
                    }
                    (Some(NumberVal::Float(acc)), &NumberVal::Float(f)) => {
                        self.acc.set(Some(NumberVal::Float(acc + f)))
                    }
                    (Some(NumberVal::Integer(acc)), &NumberVal::Float(f)) => {
                        self.acc.set(Some(NumberVal::Float((acc as f64) + f)))
                    }
                    (Some(NumberVal::Float(acc)), &NumberVal::Integer(i)) => {
                        self.acc.set(Some(NumberVal::Float(acc + (i as f64))))
                    }
                }
            } else {
                return Err(PipelineError::NotANumber{value: item.clone(), path: self.summed_value.clone()});
            }

            Ok(())
        } else if self.strict {
            Err(PipelineError::MissingValue{path: self.summed_value.clone()})
        } else {
            Ok(())
        }
    }

    fn finish(&mut self) -> Result<(), PipelineError> {
        self.output
            .ingest(JsonValue::Number(
                self.acc.get().unwrap_or(NumberVal::Integer(0)),
            ))
            .unwrap();
        self.output.finish().unwrap();
        self.acc.replace(None);

        Ok(())
    }
}

pub struct MeanStage {
    // `count` contains the `Option` information : if `count` == 0, then acc is None.
    // When `acc` is none, its value is 0, so incrementation is like a value copy,
    // and then the ingest don't have to test if acc is None before increment.
    acc: Cell<f64>,
    count: Cell<u64>,
    meaned_value: JsonPath,
    strict: bool,
    output: Box<dyn Pipeline>,
}

impl MeanStage {
    pub fn new(output: Box<dyn Pipeline>, meaned_value: JsonPath, strict: bool) -> MeanStage {
        MeanStage {
            acc: Cell::new(0f64),
            count: Cell::new(0u64),
            meaned_value,
            strict,
            output,
        }
    }

    pub fn from_args(
        output: Box<dyn Pipeline>,
        args: &[StageArg],
    ) -> Result<Box<dyn Pipeline>, String> {
        if args.len() != 1 {
            Err("mean : Wrong number of arguments.".to_string())
        } else {
            if let StageArg::Path(ref path) = args.get(0).unwrap() {
                Ok(Box::new(Self::new(output, path.clone(), false)))
            } else {
                Err("mean : Wrong type of arguments.".to_string())
            }
        }
    }
}

impl Pipeline for MeanStage {
    fn ingest(&mut self, item: JsonValue) -> Result<(), PipelineError> {
        if let Some(item) = item.select(&self.meaned_value) {
            if let JsonValue::Number(item_val) = item {
                match item_val {
                    &NumberVal::Integer(i) => self.acc.set(self.acc.get() + i as f64),
                    &NumberVal::Float(i) => self.acc.set(self.acc.get() + i),
                }
            } else {
                return Err(PipelineError::NotANumber{value: item.clone(), path: self.meaned_value.clone()});
            }

            self.count.set(self.count.get() + 1);

            Ok(())
        } else if self.strict {
            Err(PipelineError::MissingValue{path: self.meaned_value.clone()})
        } else {
            Ok(())
        }
    }

    fn finish(&mut self) -> Result<(), PipelineError> {
        if self.count.get() > 0 {
            let mean = self.acc.get() / self.count.get() as f64;

            self.output
                .ingest(JsonValue::Number(NumberVal::Float(mean)))
                .unwrap();
        }

        self.output.finish().unwrap();
        self.acc.set(0.0f64);
        self.count.set(0u64);

        Ok(())
    }
}

pub struct SelectStage {
    selected_value: JsonPath,
    output: Box<dyn Pipeline>,
}

impl SelectStage {
    pub fn new(output: Box<dyn Pipeline>, selected_value: JsonPath) -> SelectStage {
        SelectStage {
            selected_value,
            output,
        }
    }

    pub fn from_args(
        output: Box<dyn Pipeline>,
        args: &[StageArg],
    ) -> Result<Box<dyn Pipeline>, String> {
        if args.len() != 1 {
            Err("select : Wrong number of arguments.".to_string())
        } else {
            if let StageArg::Path(ref path) = args.get(0).unwrap() {
                Ok(Box::new(Self::new(output, path.clone())))
            } else {
                Err("select : Wrong type of arguments.".to_string())
            }
        }
    }
}

impl Pipeline for SelectStage {
    fn ingest(&mut self, item: JsonValue) -> Result<(), PipelineError> {
        if let Some(item) = item.select(&self.selected_value) {
            self.output.ingest(item.clone())
        } else {
            Ok(())
        }
    }

    fn finish(&mut self) -> Result<(), PipelineError> {
        self.output.finish().unwrap();

        Ok(())
    }
}
