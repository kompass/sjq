use crate::json_value::JsonValue;

pub trait Stage {
    fn ingest(&self, item: JsonValue) -> Result<(), ()>;
}

pub struct StdoutStage();

impl Stage for StdoutStage {
    fn ingest(&self, item: JsonValue) -> Result<(), ()> {
        println!("{}", item);
        Ok(())
    }
}

pub struct AddFieldStage<O: Stage> {
    key: String,
    value: JsonValue,
    output: O,
}

impl <O: Stage> AddFieldStage<O> {
    pub fn new(output: O, key: &str, value: JsonValue) -> AddFieldStage<O> {
        AddFieldStage{
            key: key.to_string(),
            value,
            output
        }
    }
}

impl <O: Stage> Stage for AddFieldStage<O> {
    fn ingest(&self, mut item: JsonValue) -> Result<(), ()> {
        if let JsonValue::Object(ref mut obj) = item {
            obj.insert(self.key.clone(), self.value.clone());

            self.output.ingest(item)
        } else {
            Err(())
        }
    }
}