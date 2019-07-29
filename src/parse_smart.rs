use std::collections::HashMap;

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
use crate::parse_and_keep::keep_json;
use crate::parse_and_throw::throw_json;


// fn array_smart_<I>(filter: JsonPath, pos: JsonPath) -> impl Parser<Input = I, Output = JsonValue>
// where
// 	I: Stream<Item = u8>,
// 	I::Error: ParseError<I::Item, I::Range, I::Position>,
// {
// 	between(token(b'['), token(b']'), sep_by::<Vec<JsonValue>, _, _>(keep_json(), lex(token(b','))).map(|v| JsonValue::Array(v)))
// }

// parser!{
//     fn array_smart[I](filter: JsonPath, pos: JsonPath)(I) -> JsonValue
//     where [I: Stream<Item = u8>]
//     {
//         array_smart_()
//     }
// }

fn object_smart_<I>(filter: JsonPath, mut pos: JsonPath) -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let field = string_val().skip(lex(token(b':'))).then(|field_name| {
		pos.push_node(field_name);

		keep_json(filter, pos)
	});

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
    fn object_smart[I](filter: JsonPath, pos: JsonPath)(I) -> JsonValue
    where [I: Stream<Item = u8>]
    {
        object_smart_()
    }
}

fn json_smart_<I>(filter: JsonPath, pos: JsonPath) -> impl Parser<Input = I, Output = JsonValue>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	if filter == pos {
		keep_json().map(|v| Some(v) )
	} else if pos.is_part(filter) {
		choice((
			throw_string().map(|_| None ),
			throw_number().map(|_| None ),
			throw_keyword().map(|_| None ),
			// array_smart(filter.clone(), pos.clone()),
			object_smart(filter, pos),
		))
	} else {
		throw_json().map(|_| None )
	}
}

parser!{
    fn json_smart[I](filter: JsonPath, pos: JsonPath)(I) -> Option<JsonValue>
    where [I: Stream<Item = u8>]
    {
        json_smart_(filter: JsonPath, pos: JsonPath)
    }
}
