mod json_value;

use std::io::stdin;
use combine::stream::ReadStream;
use combine::stream::state::State;
use combine::stream::buffered::BufferedStream;

use combine::parser::Parser;
use combine::parser::byte::{digit};
use combine::parser::item::{any, none_of, token, tokens};
use combine::parser::repeat::{many, many1};
use combine::parser::sequence::{between};
use combine::parser::combinator::from_str;

use crate::json_value::JsonValue;

fn main() {
	let stream = BufferedStream::new(State::new(ReadStream::new(stdin())), 1);

	let number_val = from_str(many1::<Vec<u8>, _>(digit())).and_then(|raw_number: String| {
		raw_number.parse::<u64>().map(|n| JsonValue::Number(n))
	});

	let string_val = between(token(b'"'), token(b'"'), from_str(many::<Vec<u8>, _>((token(b'\\').and(any()).map(|x| x.1)).or(none_of([b'"'].iter().cloned())))).map(|s: String| JsonValue::String(s) )); // TODO: Check special escaped characters

	let null_val = tokens(|l, r| *l == r, "null".into(), b"null").map(|_| JsonValue::Null );

	let true_val = tokens(|l, r| *l == r, "true".into(), b"true").map(|_| JsonValue::Boolean(true) );

	let false_val = tokens(|l, r| *l == r, "false".into(), b"false").map(|_| JsonValue::Boolean(false) );

	let mut expr = number_val.or(string_val).or(null_val).or(true_val).or(false_val);
	let value: Result<(JsonValue, _), _> = expr.easy_parse(stream);
    dbg!(value.unwrap().0);
}
