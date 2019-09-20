use sjq::parse_from_args;
use sjq::ArgStruct;
use structopt::StructOpt;
use exitfailure::ExitFailure;


// TODO: Remove as much unwrap as possible in all src/ by panics and error messages (fail fast)

fn main() -> Result<(), ExitFailure> {
    let args = ArgStruct::from_args();

    Ok(parse_from_args(args)?)
}
