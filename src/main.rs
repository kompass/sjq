mod json_value;
mod json_path;
mod parse_basics;
mod parse_and_keep;

use std::io::stdin;

use combine::stream::{ReadStream};
use combine::stream::state::State;
use combine::stream::buffered::BufferedStream;
use combine::parser::Parser;

use crate::json_value::JsonValue;
use crate::parse_and_keep::keep_json;

fn main() {
	let stream = BufferedStream::new(State::new(ReadStream::new(stdin())), 1);

	let value: Result<(JsonValue, _), _> = keep_json().easy_parse(stream);
    dbg!(value.unwrap().0);
}
