use std::collections::HashMap;
use std::iter::FromIterator;

use combine::error::ParseError;
use combine::parser::Parser;
use combine::stream::Stream;
use combine::{combine_parse_partial, combine_parser_impl, parse_mode, parser};

use combine::parser::choice::choice;
use combine::parser::item::token;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;

use crate::json_value::JsonValue;
use crate::parse_basics::{
    keyword_expr, lex, number_expr, string_expr, string_lex, token_lex, NumberVal,
};

fn keep_number<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    number_expr().map(|n: NumberVal| n.into())
}

fn keep_string<I>(max_length: usize) -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string_expr(max_length).map(|s: String| JsonValue::normalized_string(&s))
}

fn keep_keyword<I>() -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let null_val = keyword_expr("null").map(|_| JsonValue::Null);

    let true_val = keyword_expr("true").map(|_| JsonValue::Boolean(true));

    let false_val = keyword_expr("false").map(|_| JsonValue::Boolean(false));

    choice((null_val, true_val, false_val))
}

fn keep_array_<I>(max_text_length: usize) -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        token_lex('['),
        token(']'),
        sep_by::<Vec<JsonValue>, _, _>(lex(keep_json(max_text_length)), token_lex(',')),
    )
    .map(|v| JsonValue::Array(v))
}

parser! {
    fn keep_array[I](max_text_length: usize)(I) -> JsonValue
    where [I: Stream<Item = char>]
    {
        keep_array_(*max_text_length)
    }
}

fn keep_object_<I>(max_text_length: usize) -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let field = string_lex(max_text_length)
        .skip(token_lex(':'))
        .and(lex(keep_json(max_text_length)));

    let expr = between(
        token_lex('{'),
        token('}'),
        sep_by::<Vec<(String, JsonValue)>, _, _>(field, token_lex(',')),
    );

    expr.map(|v| JsonValue::Object(HashMap::from_iter(v)))
}

parser! {
    fn keep_object[I](max_text_length: usize)(I) -> JsonValue
    where [I: Stream<Item = char>]
    {
        keep_object_(*max_text_length)
    }
}

fn keep_json_<I>(max_text_length: usize) -> impl Parser<Input = I, Output = JsonValue>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        keep_string(max_text_length),
        keep_number(),
        keep_keyword(),
        keep_array(max_text_length),
        keep_object(max_text_length),
    ))
}

parser! {
    pub fn keep_json[I](max_text_length: usize)(I) -> JsonValue
    where [I: Stream<Item = char>]
    {
        keep_json_(*max_text_length)
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
        assert_eq!(keep_json(1000).parse(stream).unwrap().0, expected);
    }
}
