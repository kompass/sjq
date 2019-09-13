use std::io::{BufReader, Bytes, Read};
use std::iter::Iterator;
use unicode_normalization::{Decompositions, UnicodeNormalization};
use unicode_reader::CodePoints;

use combine::error::UnexpectedParse;
use combine::stream::buffered::BufferedStream;
use combine::stream::state::{SourcePosition, State};
use combine::stream::{IteratorStream, Positioned, Resetable, StreamErrorFor, StreamOnce};

pub struct ReadIterator<R: Read>(Option<CodePoints<Bytes<R>>>);

impl<R: Read> ReadIterator<R> {
    fn from_read(input: R) -> ReadIterator<R> {
        ReadIterator(Some(CodePoints::from(input)))
    }

    fn from_read_buffered(input: R) -> ReadIterator<BufReader<R>> {
        ReadIterator::from_read(BufReader::new(input))
    }

    pub fn from_read_buffered_normalized(input: R) -> Decompositions<ReadIterator<BufReader<R>>> {
        ReadIterator::from_read_buffered(input).nfd()
    }
}

impl<R: Read> Iterator for ReadIterator<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut codepoints) = self.0 {
            if let Some(next_or_read_err) = codepoints.next() {
                if let Ok(c) = next_or_read_err {
                    return Some(c);
                }
            }

            self.0 = None;
        }

        None
    }
}

pub struct ReadStream<R: Read>(
    BufferedStream<
        State<IteratorStream<Decompositions<ReadIterator<BufReader<R>>>>, SourcePosition>,
    >,
);

impl<R: Read> ReadStream<R> {
    pub fn from_read_buffered_normalized(input: R, buffer_size: usize) -> ReadStream<R> {
        let char_iter = ReadIterator::from_read_buffered_normalized(input);

        ReadStream(BufferedStream::new(
            State::with_positioner(IteratorStream::new(char_iter), SourcePosition::new()),
            buffer_size,
        ))
    }
}

impl<R: Read> StreamOnce for ReadStream<R> {
    type Item = char;
    type Range = char;
    type Position = SourcePosition;
    type Error = UnexpectedParse;

    fn uncons(&mut self) -> Result<Self::Item, StreamErrorFor<Self>> {
        self.0.uncons()
    }

    fn is_partial(&self) -> bool {
        self.0.is_partial()
    }
}

impl<R: Read> Resetable for ReadStream<R> {
    type Checkpoint = usize;

    fn checkpoint(&self) -> Self::Checkpoint {
        self.0.checkpoint()
    }

    fn reset(&mut self, checkpoint: Self::Checkpoint) {
        self.0.reset(checkpoint);
    }
}

impl<R: Read> Positioned for ReadStream<R> {
    fn position(&self) -> Self::Position {
        self.0.position()
    }
}
