use combine::error::ParseError;
use combine::stream::Stream;
use combine::{combine_parse_partial, combine_parser_impl, parse_mode, parser};

use combine::parser::choice::choice;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;
use combine::parser::Parser;

use crate::parse_basics::{keyword_lex, number_lex, string_lex, token_lex};

pub fn throw_number<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    number_lex().map(|_| ())
}

pub fn throw_string<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string_lex().map(|_| ())
}

pub fn throw_keyword<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let null_val = keyword_lex("null").map(|_| ());

    let true_val = keyword_lex("true").map(|_| ());

    let false_val = keyword_lex("false").map(|_| ());

    choice((null_val, true_val, false_val))
}

fn throw_array_<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        token_lex('['),
        token_lex(']'),
        sep_by::<(), _, _>(throw_json(), token_lex(',')),
    )
}

parser! {
    fn throw_array[I]()(I) -> ()
    where [I: Stream<Item = char>]
    {
        throw_array_()
    }
}

fn throw_object_<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let field = string_lex().skip(token_lex(':')).with(throw_json());

    let expr = between(
        token_lex('{'),
        token_lex('}'),
        sep_by::<(), _, _>(field, token_lex(',')),
    );

    expr
}

parser! {
    fn throw_object[I]()(I) -> ()
    where [I: Stream<Item = char>]
    {
        throw_object_()
    }
}

fn throw_json_<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
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

parser! {
    pub fn throw_json[I]()(I) -> ()
    where [I: Stream<Item = char>]
    {
        throw_json_()
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

        let stream = BufferedStream::new(State::new(IteratorStream::new(expr.chars())), 1);
        assert_eq!(throw_json().parse(stream).unwrap().0, ());
    }
}
