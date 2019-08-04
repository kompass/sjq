mod json_path;
mod json_value;
mod parse_and_keep;
mod parse_and_throw;
mod parse_basics;
mod parse_smart;
mod pipeline;
mod unicode_stream;

use std::io::stdin;
use std::io::BufReader;
use std::str::FromStr;

use combine::parser::Parser;
use combine::stream::buffered::BufferedStream;
use combine::stream::state::State;
use combine::stream::IteratorStream;

use crate::json_path::JsonPath;
use crate::json_value::JsonValue;
use crate::parse_smart::{json_smart, ParserState};
use crate::pipeline::{AddFieldStage, Stage, StdoutStage};
use crate::unicode_stream::iter_from_read;

fn main() {
    let buffered_stdin = BufReader::new(stdin());
    let char_iter = iter_from_read(buffered_stdin);
    let stream = BufferedStream::new(State::new(IteratorStream::new(char_iter)), 1);

    let pipeline: Box<dyn Stage> = Box::new(AddFieldStage::new(
        StdoutStage(),
        "pipeline_status",
        JsonValue::String("running".to_string()),
    ));

    let filter = JsonPath::from_str(".abc").unwrap();
    let state = ParserState::new(pipeline, filter);

    json_smart(state).easy_parse(stream).unwrap();
}
