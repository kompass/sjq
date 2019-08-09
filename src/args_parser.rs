use structopt::StructOpt;

/// Filter, map and aggregate huge or streaming json content
#[derive(StructOpt, Debug)]
#[structopt(
    rename_all = "kebab-case",
    max_term_width = 0,
    raw(after_help = "include_str!(\"../help/query_syntax.txt\")")
)]
pub struct ArgStruct {
    /// Writes the output into a file
    #[structopt(name = "filename", short = "o", long = "output")]
    pub output: Option<String>,

    /// If output filename specified, appends instead of overwriting previous content
    #[structopt(short, long)]
    pub append: bool,

    /// Fails if output file already exists
    #[structopt(short, long)]
    pub force_new: bool,

    /// Prettify json output
    #[structopt(short, long)]
    pub pretty: bool,

    /// Filter and pipeline query
    pub query: String,
}
