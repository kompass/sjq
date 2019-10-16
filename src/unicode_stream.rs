use std::io::Read;
use combine_elastic_buffered_stream::ElasticBufferedReadStream;

use combine::stream::{Positioned, Resetable, StreamErrorFor, StreamOnce};

pub struct ReadStream<R: Read>(
    ElasticBufferedReadStream<R>,
);

impl<R: Read> ReadStream<R> {
    pub fn from_read(input: R) -> ReadStream<R> {
        ReadStream(
            ElasticBufferedReadStream::new(input),
        )
    }
}

impl<R: Read> StreamOnce for ReadStream<R> {
    type Item = <ElasticBufferedReadStream<R> as StreamOnce>::Item;
    type Range = <ElasticBufferedReadStream<R> as StreamOnce>::Range;
    type Position = <ElasticBufferedReadStream<R> as StreamOnce>::Position;
    type Error = <ElasticBufferedReadStream<R> as StreamOnce>::Error;

    fn uncons(&mut self) -> Result<Self::Item, StreamErrorFor<Self>> {
        self.0.uncons()
    }

    fn is_partial(&self) -> bool {
        self.0.is_partial()
    }
}

impl<R: Read> Resetable for ReadStream<R> {
    type Checkpoint = <ElasticBufferedReadStream<R> as Resetable>::Checkpoint;

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
