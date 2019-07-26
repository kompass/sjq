use combine::stream::{Stream, StreamOnce};
use combine::error::ParseError;

use combine::parser::Parser;
use combine::parser::byte::{digit, spaces};
use combine::parser::item::{any, none_of, token, tokens};
use combine::parser::repeat::{many, many1, sep_by};
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

pub fn keyword_expr<I>(keyword: &'static str) -> impl Parser<Input = I, Output = ()>
	where
	I: Stream<Item = u8>,
	I::Error: ParseError<I::Item, I::Range, I::Position>,
{
	tokens(|l, r| *l == r, keyword.into(), keyword.as_bytes()).map(|_| () )
}