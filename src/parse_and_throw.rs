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


fn throw_number<I>() -> impl Parser<Input = I, Output = ()>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	number_expr().map(|_| ())
}

fn throw_string<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	string_expr().map(|_| ())
}

fn throw_keyword<I>() -> impl Parser<Input = I, Output = ()>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let null_val = keyword_expr("null").map(|_| () );

	let true_val = keyword_expr("true").map(|_| () );

	let false_val = keyword_expr("false").map(|_| () );

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
	between(token(b'['), token(b']'), sep_by::<Vec<JsonValue>, _, _>(throw_json(), lex(token(b','))).map(|_| () ))
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
	let field = string_val().skip(lex(token(b':'))).and(throw_json());

	let expr = between(token(b'{'), token(b'}'), sep_by::<Vec<()>, _, _>(field, lex(token(b','))));

	expr.map(|_| () )
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
    fn throw_json[I]()(I) -> ()
    where [I: Stream<Item = u8>]
    {
        throw_json_()
    }
}
