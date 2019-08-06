mod json_path;
mod json_value;
mod parse_and_keep;
mod parse_and_throw;
mod parse_basics;
mod parse_smart;
mod pipeline;
mod unicode_stream;

use std::io::stdin;
use std::io::BufReader;
use std::str::FromStr;

use structopt::StructOpt;

use combine::parser::Parser;
use combine::stream::buffered::BufferedStream;
use combine::stream::state::State;
use combine::stream::IteratorStream;

use crate::json_path::JsonPath;
use crate::json_value::JsonValue;
use crate::parse_smart::{json_smart, ParserState};
use crate::pipeline::{AddFieldStage, Stage, StdoutStage};
use crate::unicode_stream::ReadStream;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long)]
    output: Option<String>,

    #[structopt(short, long)]
    pretty: bool,

    query: String,
}

fn main() {
    let _ = include_str!("../Cargo.toml"); //Trigger the rebuild automatism when Cargo.toml is changed
    let args = Opt::from_args();

    let stream = ReadStream::from_read_buffered(stdin());

    let pipeline: Box<dyn Stage> = Box::new(AddFieldStage::new(
        StdoutStage(),
        "pipeline_status",
        JsonValue::String("running".to_string()),
    ));

    let filter = JsonPath::from_str(&args.query).unwrap();
    let state = ParserState::new(pipeline, filter);

    json_smart(state).easy_parse(stream).unwrap();
}
