use std::collections::HashMap;
use std::iter::FromIterator;

use combine::{parser, combine_parser_impl, combine_parse_partial, parse_mode};
use combine::stream::{Stream};
use combine::error::ParseError;

use combine::parser::Parser;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;
use combine::parser::choice::choice;

use crate::json_value::JsonValue;
use crate::parse_basics::{number_lex, string_lex, keyword_lex, token_lex};


fn keep_number<I>() -> impl Parser<Input = I, Output = JsonValue>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	number_lex().map(|n: u64| JsonValue::Number(n))
}

fn keep_string<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	string_lex().map(|s: String| JsonValue::String(s))
}

fn keep_keyword<I>() -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let null_val = keyword_lex("null").map(|_| JsonValue::Null );

	let true_val = keyword_lex("true").map(|_| JsonValue::Boolean(true) );

	let false_val = keyword_lex("false").map(|_| JsonValue::Boolean(false) );

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
	between(token_lex(b'['), token_lex(b']'), sep_by::<Vec<JsonValue>, _, _>(keep_json(), token_lex(b','))).map(|v| JsonValue::Array(v))
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
	let field = string_lex().skip(token_lex(b':')).and(keep_json());

	let expr = between(token_lex(b'{'), token_lex(b'}'), sep_by::<Vec<(String, JsonValue)>, _, _>(field, token_lex(b',')));
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
    pub fn keep_json[I]()(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        keep_json_()
    }
}

#[cfg(test)]
mod tests {
	use super::*;
	use combine::stream::IteratorStream;
	use combine::stream::state::State;
	use combine::stream::buffered::BufferedStream;

	#[test]
	fn parse_short_complex() {
		let expr = r#"{"pomme" : { "taille" :          12345,   "couleur": "jaune" },
		"random_array": [1, 2, 3    , "word" ]}"#.as_bytes().to_owned();
		let expected = JsonValue::Object([
        	("pomme".to_string(), JsonValue::Object([
               	("taille".to_string(), JsonValue::Number(12345)),
               	("couleur".to_string(), JsonValue::String("jaune".to_string()))
        	].iter().cloned().collect())),
        	("random_array".to_string(), JsonValue::Array(vec![JsonValue::Number(1), JsonValue::Number(2), JsonValue::Number(3), JsonValue::String("word".to_string())])),
    	].iter().cloned().collect());

		let stream = BufferedStream::new(State::new(IteratorStream::new(expr.into_iter())), 1);
        assert_eq!(keep_json().parse(stream).unwrap().0, expected);
	}
}