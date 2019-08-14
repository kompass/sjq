use either::Either;
use std::cell::RefCell;
use std::rc::Rc;

use combine::stream::Stream;
use combine::{combine_parse_partial, combine_parser_impl, parse_mode, parser};

use combine::parser::choice::choice;
use combine::parser::combinator::factory;
use combine::parser::repeat::sep_by;
use combine::parser::sequence::between;

use crate::filter::Filter;
use crate::json_path::JsonPath;
use crate::json_value::JsonValue;
use crate::parse_and_keep::keep_json;
use crate::parse_and_throw::throw_json;
use crate::parse_and_throw::{throw_keyword, throw_number, throw_string};
use crate::parse_basics::{string_lex, token_lex};
use crate::pipeline::Pipeline;

struct InternalState {
    pipeline: RefCell<Box<dyn Pipeline>>,
    filter: Filter,
    pos: RefCell<JsonPath>,
}

#[derive(Clone)]
pub struct ParserState(Rc<InternalState>);

impl ParserState {
    pub fn new(pipeline: Box<dyn Pipeline>, filter: Filter) -> ParserState {
        ParserState(Rc::new(InternalState {
            pipeline: RefCell::new(pipeline),
            filter,
            pos: RefCell::new(JsonPath::root()),
        }))
    }

    fn enter_node(&self, name: &str) {
        self.0.pos.borrow_mut().push_field(name);
    }

    fn exit_node(&self) {
        self.0.pos.borrow_mut().pop_field();
    }

    fn enter_array(&self) {
        self.0.pos.borrow_mut().push_index(0);
    }

    fn next_elem(&self) {
        self.0.pos.borrow_mut().inc_index();
    }

    fn exit_array(&self) {
        self.0.pos.borrow_mut().pop_index();
    }

    fn is_keeped(&self) -> bool {
        self.0.filter.is_match(&self.0.pos.borrow())
    }

    fn is_containing_keeped(&self) -> bool {
        self.0.filter.is_subpath(&self.0.pos.borrow())
    }

    fn ingest(&self, item: JsonValue) -> Result<(), String> {
        self.0.pipeline.borrow_mut().ingest(item)
    }
}

parser! {
    fn array_smart[I](state: ParserState, max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        let state_clone1 = state.clone();
        let state_clone2 = state.clone();
        let state_clone3 = state.clone();

        between(
            token_lex('[').map(move |_| { state_clone1.enter_array(); }),
            token_lex(']').map(move |_| { state_clone2.exit_array(); }),
            sep_by::<(), _, _>(json_smart(state.clone(), *max_text_length), token_lex(',').map(move |_| { state_clone3.next_elem(); }))
        )
    }
}

parser! {
    fn object_smart[I](state: ParserState, max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        let state_clone1 = state.clone();
        let state_clone2 = state.clone();

        let field = string_lex(*max_text_length).skip(token_lex(':')).then(move |field_name| {
            state_clone1.enter_node(&field_name);

            json_smart(state.clone(), *max_text_length)
        }).map(move |_| state_clone2.exit_node());

        between(
            token_lex('{'),
            token_lex('}'),
            sep_by::<(), _, _>(field, token_lex(','))
        )
    }
}

parser! {
    fn keep_json_smart[I](state: ParserState, max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        keep_json(*max_text_length).map(move |v| { state.ingest(v).unwrap(); })
    }
}

parser! {
    pub fn json_smart[I](state: ParserState, max_text_length: usize)(I) -> ()
    where [I: Stream<Item = char>]
    {
        factory(move ||
            if state.is_keeped() {
                Either::Left(keep_json_smart(state.clone(), *max_text_length))
            } else if state.is_containing_keeped() {
                Either::Right(choice((
                    throw_string(*max_text_length),
                    throw_number(),
                    throw_keyword(),
                    array_smart(state.clone(), *max_text_length),
                    object_smart(state.clone(), *max_text_length),
                )).left())
            } else {
                Either::Right(throw_json(*max_text_length).right())
            }
        )
    }
}
