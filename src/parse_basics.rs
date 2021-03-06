use lexical;
use once_cell::sync::Lazy;
use regex::Regex;

use combine::error::ParseError;
use combine::parser::byte::{alpha_num, digit, letter, spaces, bytes};
use combine::parser::choice::optional;
use combine::parser::combinator::recognize;
use combine::parser::item::{any, none_of, one_of, token};
use combine::parser::repeat::{count, count_min_max, skip_count_min_max};
use combine::parser::sequence::between;
use combine::parser::Parser;
use combine::stream::{Stream, StreamOnce};

use crate::json_value::NumberVal;

macro_rules! number_length_base_10 {
    ($n:expr) => {
        ($n as f32).log10().ceil() as usize
    };
}

// TODO : when these arithmetics will be const-compatible, use consts instead of once_cell::Lazy

// The four next consts are not the real max lengths of a valid number.
// They are there to make sure that the buffer is of sufficient size in each of the worst cases, but a valid number can't be that big.
// The converter will check itself if the numbers are not too big.
static INTEGER_PART_MAX_LENGTH: Lazy<usize> = Lazy::new(|| {
    std::cmp::max(
        std::f64::MAX_10_EXP as usize,
        number_length_base_10!(std::i64::MAX),
    )
});
static FRACTIONAL_PART_MAX_LENGTH: Lazy<usize> = Lazy::new(|| std::f64::DIGITS as usize);
static EXPONENT_MAX_LENGTH: Lazy<usize> =
    Lazy::new(|| number_length_base_10!(std::f64::MAX_10_EXP));
pub static NUMBER_MAX_LENGTH: Lazy<usize> = Lazy::new(|| {
    *&*INTEGER_PART_MAX_LENGTH + *&*FRACTIONAL_PART_MAX_LENGTH + *&*EXPONENT_MAX_LENGTH + 2
});

pub fn index_expr<I>() -> impl Parser<Input = I, Output = u64>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let expr = count_min_max::<Vec<u8>, _>(1, *&*INTEGER_PART_MAX_LENGTH, digit());

    expr.map(|v: Vec<u8>| {
        let s = String::from_utf8(v).unwrap();
        lexical::parse(&s).unwrap()
    })
}

pub fn number_expr<I>() -> impl Parser<Input = I, Output = NumberVal>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let expr = recognize::<Vec<u8>, _>((
        optional(one_of("-+".bytes())),
        skip_count_min_max(1, *&*INTEGER_PART_MAX_LENGTH, digit()),
        optional((
            token(b'.'),
            skip_count_min_max(1, *&*FRACTIONAL_PART_MAX_LENGTH, digit()),
        )),
        optional((
            one_of("eE".bytes()),
            optional(one_of("-+".bytes())),
            skip_count_min_max(1, *&*EXPONENT_MAX_LENGTH, digit()),
        )),
    ));

    expr.map(|s: Vec<u8>| {
        let float_evidences = [b'.', b'e', b'E'];
        if float_evidences.iter().any(|c| s.contains(c)) {
            NumberVal::Float(lexical::parse(&s).unwrap())
        } else {
            NumberVal::Integer(lexical::parse(&s).unwrap())
        }
    })
}

pub fn string_expr<I>(max_length: usize) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        token(b'"'),
        token(b'"'),
        count::<Vec<u8>, _>(
            max_length,
            (token(b'\\').and(any()).map(|x| x.1)).or(none_of(Some(b'"').iter().cloned())),
        ),
    ).map(|v: Vec<u8>| String::from_utf8(v).unwrap()) // TODO: Check special escaped characters
}

pub fn regex_expr<I>(max_length: usize) -> impl Parser<Input = I, Output = Regex>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let expr = between(
        token(b'/'),
        token(b'/'),
        count::<Vec<u8>, _>(
            max_length,
            (token(b'\\').and(token(b'/')).map(|x| x.1)).or(none_of(Some(b'/').iter().cloned())),
        ),
    );

    expr.map(|s| Regex::new(&String::from_utf8(s).unwrap()).unwrap())
}

pub fn ident_expr<I>(max_length: usize) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    letter()
        .and(count_min_max::<Vec<u8>, _>(
            0,
            max_length,
            alpha_num().or(token(b'_')),
        ))
        .map(move |(first, mut rest)| {
            rest.insert(0, first);
            String::from_utf8(rest).unwrap()
        })
}

pub fn keyword_expr<I>(keyword: &'static [u8]) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    bytes(keyword).map(|_| ())
}

pub fn lex<P>(p: P) -> impl Parser<Input = P::Input, Output = P::Output>
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

pub fn number_lex<I>() -> impl Parser<Input = I, Output = NumberVal>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(number_expr())
}

pub fn string_lex<I>(max_length: usize) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(string_expr(max_length))
}

pub fn keyword_lex<I>(keyword: &'static str) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(keyword_expr(keyword))
}

pub fn ident_lex<I>(max_length: usize) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(ident_expr(max_length))
}

pub fn token_lex<I>(c: char) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = u8>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(token(c)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::buffered::BufferedStream;
    use combine::stream::state::State;
    use combine::stream::IteratorStream;

    macro_rules! assert_parse_exprs {
        ($parser:expr, $exprs_and_expected:expr) => {
            for (expr, expected) in $exprs_and_expected {
                let stream =
                    BufferedStream::new(State::new(IteratorStream::new(expr.chars())), 1000);

                assert_eq!($parser.parse(stream).unwrap().0, expected);
            }
        };
    }

    #[test]
    fn parse_string() {
        let expected = vec![
            "guillotine",
            "UpPeR",
            "Text with spaces and ponctuation ? WOW, such text !",
        ];

        let exprs_and_expected: Vec<(String, _)> = expected
            .into_iter()
            .map(|e| (format!("\"{}\"", e), String::from(e)))
            .collect();

        assert_parse_exprs!(string_expr(1000), exprs_and_expected);
    }

    #[test]
    fn parse_integer() {
        let expected = vec![0i64, 1i64, 9i64, 10i64, 123456789i64, -1i64, -1345601i64];

        let exprs_and_expected: Vec<(String, _)> = expected
            .into_iter()
            .map(|e| (e.to_string(), NumberVal::Integer(e)))
            .collect();

        assert_parse_exprs!(number_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_float() {
        let expected = vec![
            0.1f64,
            // 1f64,
            1.1f64,
            10.12345f64,
            3.3333f64,
            // -1f64,
            -0.1f64,
            -134560.2f64,
        ];

        let exprs_and_expected: Vec<(String, _)> = expected
            .into_iter()
            .map(|e| (e.to_string(), NumberVal::Float(e)))
            .collect();

        assert_parse_exprs!(number_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_float_with_exponent() {}

    #[test]
    fn parse_ident() {
        let expected = vec!["abc", "askMe", "Mask", "number1"];

        let exprs_and_expected: Vec<(String, _)> = expected
            .into_iter()
            .map(|e| (e.to_string(), e.to_string()))
            .collect();

        assert_parse_exprs!(ident_expr(100), exprs_and_expected);
    }
}
