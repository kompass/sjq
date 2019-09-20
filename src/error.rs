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
    #[fail(display = "Number expected at {} but got {}", path, value)]
    NotANumber {
        value: JsonValue,
        path: JsonPath,
    }
}