mod json_value;
mod json_path;
mod parse_basics;
mod parse_and_keep;
mod parse_and_throw;

use std::io::stdin;

use combine::stream::{ReadStream};
use combine::stream::state::State;
use combine::stream::buffered::BufferedStream;
use combine::parser::Parser;

use crate::json_value::JsonValue;
use crate::parse_and_throw::throw_json;

fn main() {
	let stream = BufferedStream::new(State::new(ReadStream::new(stdin())), 1);

	let value: Result<((), _), _> = throw_json().easy_parse(stream);
    dbg!(value.unwrap().0);
}
