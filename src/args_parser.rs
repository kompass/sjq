use lexical;
use structopt::StructOpt;

use crate::parse_basics::NUMBER_MAX_LENGTH;

fn validate_max_text_length(val: String) -> Result<(), String> {
    let val: usize = lexical::parse(val);

    if val < *&*NUMBER_MAX_LENGTH {
        Err(format!(
            "--max_text_length must be bigger than {}.",
            *&*NUMBER_MAX_LENGTH
        ))
    } else {
        Ok(())
    }
}

/// Filter, map and aggregate huge or streaming json content
#[derive(StructOpt, Debug)]
#[structopt(
    rename_all = "kebab-case",
    max_term_width = 0,
    raw(after_help = "include_str!(\"../help/query_syntax.txt\")")
)]
pub struct ArgStruct {
    /// Writes the output into a file
    #[structopt(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// If output filename specified, appends instead of overwriting previous content
    #[structopt(short, long, requires = "output")]
    pub append: bool,

    /// Fails if output file already exists
    #[structopt(short, long, requires = "output")]
    pub force_new: bool,

    /// Prettify json output
    #[structopt(short, long)]
    pub pretty: bool,

    /// Max length of a string value, a field name or a regex
    #[structopt(
        short,
        long,
        default_value = "4096",
        raw(validator = "validate_max_text_length")
    )]
    pub max_text_length: usize,

    /// Filter and pipeline query
    pub query: String,
}
