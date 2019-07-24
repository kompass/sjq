mod json_value;

use std::io::stdin;
use std::collections::HashMap;
use std::iter::FromIterator;

use combine::{parser, combine_parser_impl, combine_parse_partial, parse_mode};
use combine::stream::{Stream, ReadStream, StreamOnce};
use combine::stream::state::State;
use combine::stream::buffered::BufferedStream;
use combine::error::{ParseError};

use combine::parser::Parser;
use combine::parser::byte::{digit, spaces};
use combine::parser::item::{any, none_of, token, tokens};
use combine::parser::repeat::{many, many1, sep_by};
use combine::parser::sequence::{between};
use combine::parser::combinator::from_str;
use combine::parser::choice::choice;

use crate::json_value::JsonValue;

fn lex<P>(p: P) -> impl Parser<Input = P::Input, Output = P::Output>
where
	P: Parser,
    P::Input: Stream<Item = u8>,
    <P::Input as StreamOnce>::Error: ParseError<
        <P::Input as StreamOnce>::Item,
        <P::Input as StreamOnce>::Range,
        <P::Input as StreamOnce>::Position,
	>,
{
    p.skip(spaces())
}

fn number_val<I>() -> impl Parser<Input = I, Output = JsonValue>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = many1::<Vec<u8>, _>(digit());
	let expr_str = from_str(expr);
	let json_val = expr_str.map(|s: String| JsonValue::Number(s.parse::<u64>().unwrap())); // TODO: check overflow

	json_val
}

fn string_val<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = between(token(b'"'), token(b'"'), many::<Vec<u8>, _>((token(b'\\').and(any()).map(|x| x.1)).or(none_of([b'"'].iter().cloned())))); // TODO: Check special escaped characters
	let expr_str = from_str(expr);
	let json_val = expr_str.map(|s: String| JsonValue::String(s));

	json_val
}

fn keyword_val<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let null_val = tokens(|l, r| *l == r, "null".into(), b"null").map(|_| JsonValue::Null );

	let true_val = tokens(|l, r| *l == r, "true".into(), b"true").map(|_| JsonValue::Boolean(true) );

	let false_val = tokens(|l, r| *l == r, "false".into(), b"false").map(|_| JsonValue::Boolean(false) );

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
