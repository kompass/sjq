use combine::stream::{Stream, StreamOnce};
use combine::error::ParseError;

use combine::parser::Parser;
use combine::parser::char::{digit, letter, alpha_num, spaces};
use combine::parser::item::{any, none_of, token, tokens};
use combine::parser::repeat::{many, many1};
use combine::parser::sequence::{between};


pub fn number_expr<I>() -> impl Parser<Input = I, Output = u64>
	where
	I: Stream<Item = char>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = many1::<String, _>(digit()); // TODO: accept neg and float

	expr.map(|s: String| s.parse::<u64>().unwrap()) // TODO: check overflow
}

pub fn string_expr<I>() -> impl Parser<Input = I, Output = String>
where
	I: Stream<Item = char>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = between(
		token('"'),
		token('"'),
		many::<String, _>(
			(token('\\').and(any()).map(|x| x.1))
			.or(none_of(['"'].iter().cloned()))
		)
	); // TODO: Check special escaped characters

	expr
}

pub fn ident_expr<I>() -> impl Parser<Input = I, Output = String>
	where
	I: Stream<Item = char>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	let expr = letter().and(many::<String, _>(alpha_num()))
		.map(|(first, rest)| { let mut val = rest.clone(); val.insert(0, first); val });

	expr
}

pub fn keyword_expr<I>(keyword: &'static str) -> impl Parser<Input = I, Output = ()>
	where
	I: Stream<Item = char>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	tokens(|l, r| l == r, keyword.into(), keyword.chars()).map(|_| () )
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

pub fn number_lex<I>() -> impl Parser<Input = I, Output = u64>
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

pub fn ident_lex<I>() -> impl Parser<Input = I, Output = String>
    where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(ident_expr())
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
    lex(token(c)).map(|_| () )
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
				let stream = BufferedStream::new(State::new(IteratorStream::new(expr.chars())), 1);

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

    	let exprs_and_expected: Vec<(String, _)> =
    		expected
    		.into_iter()
    		.map(|e| (
    			format!("\"{}\"", e),
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

    	let exprs_and_expected: Vec<(String, _)> =
    		expected
    		.into_iter()
    		.map(|e| (
    			e.to_string(),
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

    	let exprs_and_expected: Vec<(String, _)> =
    		expected
    		.into_iter()
    		.map(|e| (
    			e.to_string(),
    			e.to_string()
    		)).collect();

    	assert_parse_exprs!(ident_expr(), exprs_and_expected);
    }
}
