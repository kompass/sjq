use std::ops::Range;

use combine::char::alpha_num;
use combine::error::ParseError;
use combine::parser::choice::choice;
use combine::parser::choice::optional;
use combine::parser::combinator::attempt;
use combine::parser::combinator::not_followed_by;
use combine::parser::item::eof;
use combine::parser::item::token;
use combine::parser::repeat::many;
use combine::parser::repeat::many1;
use combine::parser::repeat::sep_by1;
use combine::parser::sequence::between;
use combine::parser::Parser;
use combine::stream::state::State;
use combine::stream::Stream;

use crate::filter::*;
use crate::json_path::{JsonPath, JsonPathStep};
use crate::parse_basics::{
    ident_expr, ident_lex, index_expr, lex, number_lex, regex_expr, string_expr, string_lex,
    token_lex, NumberVal,
};
use crate::pipeline::*;
use crate::pipeline_builder::StageArg;

fn path_parser<I>(max_text_length: usize) -> impl Parser<Input = I, Output = JsonPath>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let field_path_expr = token('.')
        .with(string_expr(max_text_length).or(ident_expr(max_text_length)))
        .message("field_path_expr")
        .map(|field_name| JsonPathStep::Field(field_name));

    let index_path_expr = between(token('['), token(']'), index_expr())
        .message("index_path_expr")
        .map(|array_index| JsonPathStep::Index(array_index));

    let path_step_expr = field_path_expr.or(index_path_expr);

    choice((
        attempt(token('.').skip(not_followed_by(alpha_num()))).map(|_| JsonPath::root()),
        attempt(many1::<Vec<_>, _>(path_step_expr)).map(|v| JsonPath::new(v)),
    ))
    .message("path_parser")
}

fn filter_parser<I>(max_text_length: usize) -> impl Parser<Input = I, Output = Filter>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let array_filter_expr_internal = index_expr()
        .and(optional(choice((
            many::<Vec<_>, _>(token(',').with(index_expr())).map(|v| ArrayFilter::OneOf(v)),
            token(':').with(index_expr()).map(|right_bound| {
                ArrayFilter::Range(Range {
                    start: 0,
                    end: right_bound,
                })
            }),
        ))))
        .message("array_filter_expr_internal")
        .map(|(num, maybe_rest)| {
            if let Some(array_filter) = maybe_rest {
                match array_filter {
                    ArrayFilter::OneOf(mut v) => {
                        v.push(num);

                        ArrayFilter::OneOf(v)
                    }
                    ArrayFilter::Range(mut r) => {
                        r.start = num;

                        ArrayFilter::Range(r)
                    }
                    ArrayFilter::ExactValue(_) => unreachable!(),
                }
            } else {
                ArrayFilter::ExactValue(num)
            }
        });

    let array_filter_expr = between(token('['), token(']'), array_filter_expr_internal)
        .message("array_filter_expr")
        .map(|array_filter| FilterPart::Array(array_filter));

    let branch_filter_expr = token('.')
        .with(choice((
            string_expr(max_text_length)
                .map(|branch_name| FilterPart::Branch(BranchFilter::TextMatch(branch_name))),
            ident_expr(max_text_length)
                .message("ident_expr")
                .map(|branch_name| FilterPart::Branch(BranchFilter::TextMatch(branch_name))),
            regex_expr(max_text_length)
                .map(|reg| FilterPart::Branch(BranchFilter::RegexMatch(reg))),
        )))
        .message("branch_filter_expr");

    let filter_part_expr = array_filter_expr.or(branch_filter_expr);

    let filter_expr = attempt(token('.').skip(eof()).map(|_| vec![]))
        .or(many::<Vec<_>, _>(filter_part_expr))
        .map(|v| {
            if v.is_empty() {
                Filter::All
            } else {
                Filter::Parts(v)
            }
        });

    sep_by1::<Vec<_>, _, _>(filter_expr, token_lex(','))
        .map(|mut v| {
            if v.len() == 1 {
                v.pop().unwrap()
            } else {
                Filter::Union(v)
            }
        })
        .message("filter_parser")
}

fn stage_parser<I>(
    max_text_length: usize,
) -> impl Parser<Input = I, Output = (String, Vec<StageArg>)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    ident_lex(max_text_length).and(many::<Vec<StageArg>, _>(choice((
        number_lex().map(|n: NumberVal| StageArg::Number(n)),
        string_lex(max_text_length).map(|s: String| StageArg::String(s)),
        lex(path_parser(max_text_length)).map(|p: JsonPath| StageArg::Path(p)),
    ))))
}

pub fn parse_query<'a>(
    max_text_length: usize,
    output: Box<dyn Pipeline>,
    query: &str,
) -> Result<(Filter, Box<dyn Pipeline + 'a>), String> {
    let mut parser = filter_parser(max_text_length).and(many::<Vec<_>, _>(
        token_lex('|').with(stage_parser(max_text_length)),
    ));

    let (filter, stages) = parser
        .easy_parse(State::new(query))
        .map(|(filter, _)| filter)
        .map_err(|err| format!("{}", err))
        .unwrap();

    let mut pipeline = output;

    for (stage_ident, args) in stages.iter().rev() {
        pipeline = match stage_ident.as_str() {
            "add_field" => AddFieldStage::from_args(pipeline, &args),
            "mean" => MeanStage::from_args(pipeline, &args),
            "sum" => SumStage::from_args(pipeline, &args),
            &_ => Err("Unknown stage name.".to_string()),
        }
        .unwrap();
    }

    Ok((filter, pipeline)) // TODO: gérer les erreurs de syntaxe en virant l'unwrap
}
