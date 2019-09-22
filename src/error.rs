use failure::Fail;

use crate::json_path::JsonPath;
use crate::json_value::JsonValue;

#[derive(Debug, Fail)]
pub enum InitError {
    #[fail(display = "unable to open the file {}", filename)]
    UnableToOpenFile { filename: String },

    #[fail(display = "syntax error in the query at position {}", position)]
    WrongQuerySyntax { position: i32 },

    #[fail(display = "unknown stage : {}", unknown_name)]
    StageUnknownName { unknown_name: String },

    #[fail(
        display = "wrong number of arguments for stage {}, expected {}, got {}",
        stage_name, expected, got
    )]
    StageWrongNumberArgs {
        stage_name: String,
        expected: usize,
        got: usize,
    },

    #[fail(
        display = "argument #{} of stage {} is of wrong type",
        arg_pos, stage_name
    )]
    StageWrongArgType { stage_name: String, arg_pos: usize },
}

#[derive(Debug, Fail)]
pub enum PipelineError {
    #[fail(display = "number expected at {} but got {}", path, value)]
    NotANumber { value: JsonValue, path: JsonPath },

    #[fail(display = "object expected but got {}", value)]
    NotAnObject { value: JsonValue },

    #[fail(display = "missing value at {}", path)]
    MissingValue { path: JsonPath },

    #[fail(display = "unable to write to output")]
    UnableToWriteOuptut,
}
