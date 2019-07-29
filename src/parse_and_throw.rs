use combine::{parser, combine_parser_impl, combine_parse_partial, parse_mode};
use combine::stream::{Stream, ReadStream};
use combine::stream::state::State;
use combine::stream::buffered::BufferedStream;
use combine::error::ParseError;

use combine::parser::Parser;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;

use combine::parser::choice::choice;

use crate::json_value::JsonValue;
use crate::parse_basics::{number_lex, string_lex, keyword_lex, token_lex};


fn throw_number<I>() -> impl Parser<Input = I, Output = ()>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	number_lex().map(|_| ())
}

fn throw_string<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	string_lex().map(|_| ())
}

fn throw_keyword<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let null_val = keyword_lex("null").map(|_| () );

	let true_val = keyword_lex("true").map(|_| () );

	let false_val = keyword_lex("false").map(|_| () );

	choice((
		null_val,
		true_val,
		false_val,
	))
}

fn throw_array_<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	between(token_lex(b'['), token_lex(b']'), sep_by::<Vec<()>, _, _>(throw_json(), token_lex(b',')).map(|_| () ))
}

parser!{
    fn throw_array[I]()(I) -> ()
    where [I: Stream<Item = u8>]
    {
        throw_array_()
    }
}

fn throw_object_<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let field = string_lex().skip(token_lex(b':')).and(throw_json()).map(|_| () );

	let expr = between(token_lex(b'{'), token_lex(b'}'), sep_by::<Vec<()>, _, _>(field, token_lex(b',')).map(|_| () ));

	expr
}

parser!{
    fn throw_object[I]()(I) -> ()
    where [I: Stream<Item = u8>]
    {
        throw_object_()
    }
}

fn throw_json_<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	choice((
		throw_string(),
		throw_number(),
		throw_keyword(),
		throw_array(),
		throw_object(),
	))
}

parser!{
    pub fn throw_json[I]()(I) -> ()
    where [I: Stream<Item = u8>]
    {
        throw_json_()
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

		let stream = BufferedStream::new(State::new(IteratorStream::new(expr.into_iter())), 1);
        assert_eq!(throw_json().parse(stream).unwrap().0, ());
	}
}