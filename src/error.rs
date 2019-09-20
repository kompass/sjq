use failure::{Error, Fail};

#[derive(Debug, Fail)]
pub enum InitError {
    #[fail(display = "unable to open the file {}", filename)]
    UnableToOpenFile {
        filename: String,
    },
    #[fail(display = "syntax error in the query at position {}", position)]
    WrongQuerySyntax {
        position: usize,
    }
}
