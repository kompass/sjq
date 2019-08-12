use std::collections::HashMap;
use std::iter::FromIterator;

use combine::error::ParseError;
use combine::stream::Stream;
use combine::{combine_parse_partial, combine_parser_impl, parse_mode, parser};

use combine::parser::choice::choice;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;
use combine::parser::Parser;

use crate::json_value::JsonValue;
use crate::parse_basics::{NumberVal, keyword_lex, number_lex, string_lex, token_lex};

fn keep_number<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    number_lex().map(|n: NumberVal| n.into())
}

fn keep_string<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string_lex().map(|s: String| JsonValue::String(s))
}

fn keep_keyword<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let null_val = keyword_lex("null").map(|_| JsonValue::Null);

    let true_val = keyword_lex("true").map(|_| JsonValue::Boolean(true));

    let false_val = keyword_lex("false").map(|_| JsonValue::Boolean(false));

    choice((null_val, true_val, false_val))
}

fn keep_array_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        token_lex('['),
        token_lex(']'),
        sep_by::<Vec<JsonValue>, _, _>(keep_json(), token_lex(',')),
    )
    .map(|v| JsonValue::Array(v))
}

parser! {
    fn keep_array[I]()(I) -> JsonValue
    where [I: Stream<Item = char>]
    {
        keep_array_()
    }
}

fn keep_object_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let field = string_lex().skip(token_lex(':')).and(keep_json());

    let expr = between(
        token_lex('{'),
        token_lex('}'),
        sep_by::<Vec<(String, JsonValue)>, _, _>(field, token_lex(',')),
    );

    expr.map(|v| JsonValue::Object(HashMap::from_iter(v)))
}

parser! {
    fn keep_object[I]()(I) -> JsonValue
    where [I: Stream<Item = char>]
    {
        keep_object_()
    }
}

fn keep_json_<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
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

parser! {
    pub fn keep_json[I]()(I) -> JsonValue
    where [I: Stream<Item = char>]
    {
        keep_json_()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::buffered::BufferedStream;
    use combine::stream::state::State;
    use combine::stream::IteratorStream;

    #[test]
    fn parse_short_complex() {
        let expr = r#"{"pomme" : { "taille" :          12345,   "couleur": "jaune" },
        "random_array": [1, 2, 3    , "word" ]}"#;
        let expected = JsonValue::Object(
            [
                (
                    "pomme".to_string(),
                    JsonValue::Object(
                        [
                            ("taille".to_string(), JsonValue::Integer(12345)),
                            (
                                "couleur".to_string(),
                                JsonValue::String("jaune".to_string()),
                            ),
                        ]
                        .iter()
                        .cloned()
                        .collect(),
                    ),
                ),
                (
                    "random_array".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::Integer(1),
                        JsonValue::Integer(2),
                        JsonValue::Integer(3),
                        JsonValue::String("word".to_string()),
                    ]),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        );

        let stream = BufferedStream::new(State::new(IteratorStream::new(expr.chars())), 1000);
        assert_eq!(keep_json().parse(stream).unwrap().0, expected);
    }
}
