mod json_path;
mod json_value;
mod parse_and_keep;
mod parse_and_throw;
mod parse_basics;
mod unicode_stream;

use std::io::stdin;

use combine::parser::Parser;
use combine::stream::buffered::BufferedStream;
use combine::stream::state::State;
use combine::stream::IteratorStream;

use crate::json_value::JsonValue;
use crate::parse_and_keep::keep_json;
use crate::unicode_stream::iter_from_read;

fn main() {
	let stream = BufferedStream::new(State::new(IteratorStream::new(iter_from_read(stdin()))), 1);

	let value: Result<(JsonValue, _), _> = keep_json().easy_parse(stream);
    dbg!(value.unwrap().0);
}
