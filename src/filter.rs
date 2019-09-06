use regex::Regex;
use std::ops::Range;

use crate::json_path::{JsonPath, JsonPathStep};

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
    fn is_match(&self, pos_part: &JsonPathStep) -> bool {
        match self {
            FilterPart::Branch(ref branch_filter) => {
                if let JsonPathStep::Field(ref branch_name) = pos_part {
                    branch_filter.is_match(branch_name)
                } else {
                    false
                }
            }
            FilterPart::Array(ref array_filter) => {
                if let JsonPathStep::Index(array_index) = pos_part {
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

                for (filter_part, path_step) in zipped {
                    if !filter_part.is_match(path_step) {
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
}
