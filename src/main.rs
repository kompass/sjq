use sjq::parse_from_args;
use sjq::ArgStruct;
use structopt::StructOpt;

// TODO: Remove as much unwrap as possible in all src/ by panics and error messages (fail fast)

fn main() {
    let args = ArgStruct::from_args();

    parse_from_args(args).unwrap();
}
