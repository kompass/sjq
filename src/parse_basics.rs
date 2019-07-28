use combine::stream::{Stream, StreamOnce};
use combine::error::ParseError;

use combine::parser::Parser;
use combine::parser::byte::{digit, letter, alpha_num, spaces};
use combine::parser::item::{any, none_of, token, tokens};
use combine::parser::repeat::{many, many1};
use combine::parser::sequence::{between};
use combine::parser::combinator::from_str;

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

pub fn number_expr<I>() -> impl Parser<Input = I, Output = u64>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = many1::<Vec<u8>, _>(digit()); // TODO: accept neg and float

	from_str(expr).map(|s: String| s.parse::<u64>().unwrap()) // TODO: check overflow
}

pub fn string_expr<I>() -> impl Parser<Input = I, Output = String>
where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = between(
		token(b'"'),
		token(b'"'),
		many::<Vec<u8>, _>(
			(token(b'\\').and(any()).map(|x| x.1))
			.or(none_of([b'"'].iter().cloned()))
		)
	); // TODO: Check special escaped characters

	from_str(expr)
}

pub fn ident_expr<I>() -> impl Parser<Input = I, Output = String>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = letter().and(many::<Vec<_>, _>(alpha_num()))
		.map(|(first, rest)| { let mut val = rest.clone(); val.insert(0, first); val });

	from_str(expr)
}

pub fn keyword_expr<I>(keyword: &'static str) -> impl Parser<Input = I, Output = ()>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	tokens(|l, r| *l == r, keyword.into(), keyword.as_bytes()).map(|_| () )
}

#[cfg(test)]
mod tests {
	use super::*;
	use combine::stream::IteratorStream;
	use combine::stream::state::State;
	use combine::stream::buffered::BufferedStream;

	macro_rules! assert_parse_exprs {
		($parser:expr, $exprs_and_expected:expr) => {
			for (expr, expected) in $exprs_and_expected {
				let stream = BufferedStream::new(State::new(IteratorStream::new(expr.into_iter())), 1);

				assert_eq!($parser.parse(stream).unwrap().0, expected);
			}
		};
	}

    #[test]
    fn parse_string() {
    	let expected = vec![
    		"guillotine",
    		"UpPeR",
    		"Let's make revolution!",
    	];

    	let exprs_and_expected: Vec<(Vec<u8>, _)> =
    		expected
    		.into_iter()
    		.map(|e| (
    			format!("\"{}\"", e).as_bytes().to_owned(),
    			String::from(e)
    		)).collect();

        assert_parse_exprs!(string_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_number() {
    	let expected = vec![
    		0u64,
    		1u64,
    		9u64,
    		10u64,
    		123456789u64,
    	];

    	let exprs_and_expected: Vec<(Vec<u8>, _)> =
    		expected
    		.into_iter()
    		.map(|e| (
    			e.to_string().as_bytes().to_owned(),
    			e
    		)).collect();

        assert_parse_exprs!(number_expr(), exprs_and_expected);
    }

    #[test]
    fn parse_ident() {
    	let expected = vec![
    		"abc",
    		"askMe",
    		"Mask",
    		"number1",
    	];

    	let exprs_and_expected: Vec<(Vec<u8>, _)> =
    		expected
    		.into_iter()
    		.map(|e| (
    			e.to_string().as_bytes().to_owned(),
    			e.to_string()
    		)).collect();

    	assert_parse_exprs!(ident_expr(), exprs_and_expected);
    }
}
