use regex::Regex;
use lexical;

use combine::error::ParseError;
use combine::stream::{Stream, StreamOnce};

use combine::parser::char::{alpha_num, digit, letter, spaces, string};
use combine::parser::choice::optional;
use combine::parser::item::one_of;
use combine::parser::combinator::recognize;
use combine::parser::item::{any, none_of, token};
use combine::parser::repeat::{many, many1, skip_many1};
use combine::parser::sequence::between;
use combine::parser::Parser;

use crate::json_value::JsonValue;

pub fn index_expr<I>() -> impl Parser<Input = I, Output = u64>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let expr = many1::<String, _>(digit());

    expr.map(|s: String| s.parse::<u64>().unwrap())
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberVal {
    Integer(i64),
    Float(f64),
}

impl Into<JsonValue> for NumberVal {
    fn into(self) -> JsonValue {
        match self {
            NumberVal::Integer(n) => JsonValue::Integer(n),
            NumberVal::Float(f) => JsonValue::Float(f),
        }
    }
}

pub fn number_expr<I>() -> impl Parser<Input = I, Output = NumberVal>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let expr = recognize::<String, _>((
        optional(one_of("-+".chars())),
        skip_many1(digit()),
        optional((token('.'), skip_many1(digit()))),
        optional((
            one_of("eE".chars()),
            optional(one_of("-+".chars())),
            skip_many1(digit())
        )),
    ));

    expr.map(|s: String| {
        dbg!(&s);
        let float_evidences = ['.', 'e', 'E'];
        if s.contains(float_evidences.as_ref()) {
            NumberVal::Float(lexical::try_parse(&s).unwrap()) // TODO: Let the user choose try_parse_lossy or not
        } else {
            NumberVal::Integer(lexical::try_parse(&s).unwrap())
        }
    })
}

pub fn string_expr<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        token('"'),
        token('"'),
        many::<String, _>(
            (token('\\').and(any()).map(|x| x.1)).or(none_of(Some('"').iter().cloned())),
        ),
    ) // TODO: Check special escaped characters
}

pub fn regex_expr<I>() -> impl Parser<Input = I, Output = Regex>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let expr = between(
        token('/'),
        token('/'),
        many::<String, _>(
            (token('\\').and(token('/')).map(|x| x.1)).or(none_of(Some('/').iter().cloned())),
        ),
    );

    expr.map(|s| Regex::new(&s).unwrap())
}

pub fn ident_expr<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    letter()
        .and(many::<String, _>(alpha_num().or(token('_'))))
        .map(move |(first, mut rest)| {
            rest.insert(0, first);
            rest
        })
}

pub fn keyword_expr<I>(keyword: &'static str) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string(keyword).map(|_| ())
}

fn lex<P>(p: P) -> impl Parser<Input = P::Input, Output = P::Output>
where
    P: Parser,
    P::Input: Stream<Item = char>,
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
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(number_expr())
}

pub fn string_lex<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(string_expr())
}

pub fn keyword_lex<I>(keyword: &'static str) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(keyword_expr(keyword))
}

pub fn token_lex<I>(c: char) -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
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
                let stream = BufferedStream::new(State::new(IteratorStream::new(expr.chars())), 100);

                assert_eq!($parser.parse(stream).unwrap().0, expected);
            }
        };
    }

    #[test]
    fn parse_string() {
        let expected = vec!["guillotine", "UpPeR", "Text with spaces and ponctuation ? WOW, such text !"];

        let exprs_and_expected: Vec<(String, _)> = expected
            .into_iter()
            .map(|e| (format!("\"{}\"", e), String::from(e)))
            .collect();

        assert_parse_exprs!(string_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_integer() {
        let expected = vec![0i64, 1i64, 9i64, 10i64, 123456789i64, -1i64, -1345601i64];

        let exprs_and_expected: Vec<(String, _)> =
            expected.into_iter().map(|e| (e.to_string(), NumberVal::Integer(e))).collect();

        assert_parse_exprs!(number_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_float() {
        let expected = vec![0.1f64, 1f64, 1.1f64, 10.12345f64, 3.3333f64, -1f64, -0.1f64, -134560.2f64];

        let exprs_and_expected: Vec<(String, _)> =
            expected.into_iter().map(|e| (e.to_string(), NumberVal::Float(e))).collect();

        assert_parse_exprs!(number_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_float_with_exponent() {

    }

    #[test]
    fn parse_ident() {
        let expected = vec!["abc", "askMe", "Mask", "number1"];

        let exprs_and_expected: Vec<(String, _)> = expected
            .into_iter()
            .map(|e| (e.to_string(), e.to_string()))
            .collect();

        assert_parse_exprs!(ident_expr(), exprs_and_expected);
    }
}
