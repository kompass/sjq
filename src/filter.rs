use regex::Regex;
use std::ops::Range;
use std::str::FromStr;

use combine::attempt;
use combine::error::ParseError;
use combine::parser::Parser;
use combine::stream::Stream;

use combine::parser::choice::{choice, optional};
use combine::parser::item::eof;
use combine::parser::item::token;
use combine::parser::repeat::{many, sep_by1};
use combine::parser::sequence::between;

use crate::json_path::{JsonPath, JsonPathStage};
use crate::parse_basics::{ident_expr, number_expr, regex_expr, string_expr, token_lex};

pub enum BranchFilter {
    TextMatch(String),
    RegexMatch(Regex),
}

impl BranchFilter {
    fn is_match(&self, branch_name: &str) -> bool {
        match self {
            BranchFilter::TextMatch(ref text) => branch_name == text,
            BranchFilter::RegexMatch(ref reg) => reg.is_match(branch_name),
        }
    }
}

pub enum ArrayFilter {
    ExactValue(u64),
    OneOf(Vec<u64>),
    Range(Range<u64>),
}

impl ArrayFilter {
    fn is_match(&self, array_index: u64) -> bool {
        match self {
            ArrayFilter::ExactValue(val) => array_index == *val,
            ArrayFilter::OneOf(arr) => arr.contains(&array_index),
            ArrayFilter::Range(range) => range.contains(&array_index),
        }
    }
}

pub enum FilterPart {
    Branch(BranchFilter),
    Array(ArrayFilter),
}

impl FilterPart {
    fn is_match(&self, pos_part: &JsonPathStage) -> bool {
        match self {
            FilterPart::Branch(ref branch_filter) => {
                if let JsonPathStage::Node(ref branch_name) = pos_part {
                    branch_filter.is_match(branch_name)
                } else {
                    false
                }
            }
            FilterPart::Array(ref array_filter) => {
                if let JsonPathStage::Index(array_index) = pos_part {
                    array_filter.is_match(*array_index)
                } else {
                    false
                }
            }
        }
    }
}

pub enum Filter {
    All,
    Parts(Vec<FilterPart>),
    Union(Vec<Filter>),
}

impl Filter {
    fn compare(&self, pos: &JsonPath, subpath: bool) -> bool {
        match self {
            Filter::All => true,
            Filter::Parts(ref parts) => {
                if !subpath && pos.len() < parts.len() {
                    return false;
                }

                let zipped = parts.iter().zip(pos.iter());

                for (filter_part, path_stage) in zipped {
                    if !filter_part.is_match(path_stage) {
                        return false;
                    }
                }

                true
            }
            Filter::Union(ref filters) => filters.iter().any(|filter| filter.is_match(pos)),
        }
    }

    pub fn is_match(&self, pos: &JsonPath) -> bool {
        self.compare(pos, false)
    }

    pub fn is_subpath(&self, pos: &JsonPath) -> bool {
        self.compare(pos, true)
    }

    fn parser<I>() -> impl Parser<Input = I, Output = Filter>
    where
        I: Stream<Item = char>,
        I::Error: ParseError<I::Item, I::Range, I::Position>,
    {
        let array_filter_expr_internal = number_expr()
            .and(optional(choice((
                many::<Vec<_>, _>(token(',').with(number_expr())).map(|v| ArrayFilter::OneOf(v)),
                token(':').with(number_expr()).map(|right_bound| {
                    ArrayFilter::Range(Range {
                        start: 0,
                        end: right_bound,
                    })
                }),
            ))))
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
            .map(|array_filter| FilterPart::Array(array_filter));

        let branch_filter_expr = token('.').with(
            string_expr()
                .or(ident_expr())
                .map(|branch_name| FilterPart::Branch(BranchFilter::TextMatch(branch_name)))
                .or(regex_expr().map(|reg| FilterPart::Branch(BranchFilter::RegexMatch(reg)))),
        );

        let filter_part_expr = array_filter_expr.or(branch_filter_expr);

        let filter_expr = attempt(token('.').skip(eof()).map(|_| vec![]))
            .or(many::<Vec<_>, _>(filter_part_expr).skip(eof()))
            .map(|v| {
                if v.is_empty() {
                    Filter::All
                } else {
                    Filter::Parts(v)
                }
            });

        sep_by1::<Vec<_>, _, _>(filter_expr, token_lex(',')).map(|mut v| {
            if v.len() == 1 {
                v.pop().unwrap()
            } else {
                Filter::Union(v)
            }
        })
    }
}

impl FromStr for Filter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parser()
            .easy_parse(s)
            .map(|(filter, _)| filter)
            .map_err(|err| format!("{}", err))
    }
}
