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

pub fn throw_string<I>(max_length: usize) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string_lex(max_length).map(|_| ())
}

pub fn throw_keyword<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let null_val = keyword_lex("null");

    let true_val = keyword_lex("true");

    let false_val = keyword_lex("false");

    choice((null_val, true_val, false_val))
}

fn throw_array_<I>(max_text_length: usize) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        token_lex('['),
        token_lex(']'),
        sep_by::<(), _, _>(throw_json(max_text_length), token_lex(',')),
    )
}

parser! {
    fn throw_array[I](max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        throw_array_(*max_text_length)
    }
}

fn throw_object_<I>(max_text_length: usize) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let field = string_lex(max_text_length)
        .skip(token_lex(':'))
        .with(throw_json(max_text_length));

    between(
        token_lex('{'),
        token_lex('}'),
        sep_by::<(), _, _>(field, token_lex(',')),
    )
}

parser! {
    fn throw_object[I](max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        throw_object_(*max_text_length)
    }
}

fn throw_json_<I>(max_text_length: usize) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        throw_string(max_text_length),
        throw_number(),
        throw_keyword(),
        throw_array(max_text_length),
        throw_object(max_text_length),
    ))
}

parser! {
    pub fn throw_json[I](max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        throw_json_(*max_text_length)
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

        let stream = BufferedStream::new(State::new(IteratorStream::new(expr.chars())), 1000);
        assert_eq!(throw_json().parse(stream).unwrap().0, ());
    }
}
