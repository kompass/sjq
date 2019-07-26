mod json_value;
mod json_path;
mod parse_basics;

use std::io::stdin;
use std::collections::HashMap;
use std::iter::FromIterator;

use combine::{parser, combine_parser_impl, combine_parse_partial, parse_mode};
use combine::stream::{Stream, ReadStream};
use combine::stream::state::State;
use combine::stream::buffered::BufferedStream;
use combine::error::ParseError;

use combine::parser::Parser;
use combine::parser::item::token;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;

use combine::parser::choice::choice;

use crate::json_value::JsonValue;
use crate::parse_basics::{lex, number_expr, string_expr, keyword_expr};


fn number_val<I>() -> impl Parser<Input = I, Output = JsonValue>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	number_expr().map(|n: u64| JsonValue::Number(n))
}

fn string_val<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	string_expr().map(|s: String| JsonValue::String(s))
}

fn keyword_val<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let null_val = keyword_expr("null").map(|_| JsonValue::Null );

	let true_val = keyword_expr("true").map(|_| JsonValue::Boolean(true) );

	let false_val = keyword_expr("false").map(|_| JsonValue::Boolean(false) );

	choice((
		null_val,
		true_val,
		false_val,
	))
}

fn array_val_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	between(token(b'['), token(b']'), sep_by::<Vec<JsonValue>, _, _>(json_val(), lex(token(b','))).map(|v| JsonValue::Array(v)))
}

parser!{
    fn array_val[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        array_val_()
    }
}

fn object_val_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let field = string_val().skip(lex(token(b':'))).and(json_val());
	let field_tuple = field.map(|(k, v)| {
		if let JsonValue::String(s) = k {
			(s, v)
		} else {
			unreachable!()
		}
	});

	let expr = between(token(b'{'), token(b'}'), sep_by::<Vec<(String, JsonValue)>, _, _>(field_tuple, lex(token(b','))));
	let value = expr.map(|v| JsonValue::Object(HashMap::from_iter(v)));

	value
}

parser!{
    fn object_val[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        object_val_()
    }
}

fn json_val_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	choice((
		string_val(),
		number_val(),
		keyword_val(),
		array_val(),
		object_val(),
	))
}

parser!{
    fn json_val[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        json_val_()
    }
}

fn main() {
	let stream = BufferedStream::new(State::new(ReadStream::new(stdin())), 1);

	let value: Result<(JsonValue, _), _> = json_val().easy_parse(stream);
    dbg!(value.unwrap().0);
}
