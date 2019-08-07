use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ArgStruct {
    #[structopt(short, long)]
    pub output: Option<String>,

    #[structopt(short, long)]
    pub pretty: bool,

    pub query: String,
}
