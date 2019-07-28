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


fn keep_number<I>() -> impl Parser<Input = I, Output = JsonValue>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	number_expr().map(|n: u64| JsonValue::Number(n))
}

fn keep_string<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	string_expr().map(|s: String| JsonValue::String(s))
}

fn keep_keyword<I>() -> impl Parser<Input = I, Output = JsonValue>
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

fn keep_array_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	between(token(b'['), token(b']'), sep_by::<Vec<JsonValue>, _, _>(keep_json(), lex(token(b','))).map(|v| JsonValue::Array(v)))
}

parser!{
    fn keep_array[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        keep_array_()
    }
}

fn keep_object_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let field = string_val().skip(lex(token(b':'))).and(keep_json());
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
    fn keep_object[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        keep_object_()
    }
}

fn keep_json_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	choice((
		keep_string(),
		keep_number(),
		keep_keyword(),
		keep_array(),
		keep_object(),
	))
}

parser!{
    fn keep_json[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        keep_json_()
    }
}
