use std::io::Read;
use unicode_reader::CodePoints;

pub fn iter_from_read<R: Read>(input: R) -> impl std::iter::Iterator<Item = char> {
    CodePoints::from(input)
        .take_while(|r| r.is_ok())
        .map(|r| r.unwrap())
}
