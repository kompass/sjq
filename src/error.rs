use failure::Fail;

use crate::json_value::JsonValue;
use crate::json_path::JsonPath;

#[derive(Debug, Fail)]
pub enum InitError {
    #[fail(display = "unable to open the file {}", filename)]
    UnableToOpenFile {
        filename: String,
    },

    #[fail(display = "syntax error in the query at position {}", position)]
    WrongQuerySyntax {
        position: i32,
    }
}

#[derive(Debug, Fail)]
pub enum PipelineError {
    #[fail(display = "number expected at {} but got {}", path, value)]
    NotANumber {
        value: JsonValue,
        path: JsonPath,
    },
    #[fail(display = "object expected but got {}", value)]
    NotAnObject{
        value: JsonValue,
    },
    #[fail(display = "missing value at {}", path)]
    MissingValue {
        path: JsonPath,
    }
}