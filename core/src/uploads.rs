use std::{
    pin::Pin,
    task::{Context, Poll},
};

use eva::{bytes::Bytes, data};

use futures::stream::Stream;

#[data(error)]
pub enum AbortReason {
    #[display("the user uploaded more than expected")]
    Overuploading,
    #[display("{_0}")]
    Other(String),
}

#[data]
pub enum Chunk {
    Data(Bytes),
    Aborted(AbortReason),
}

type BoxedStream<'a> = Pin<Box<dyn Stream<Item = Chunk> + Send + Sync + 'a>>;

pub struct UploadStream<'a> {
    stream: BoxedStream<'a>,
    left: Option<usize>,
}

impl Unpin for UploadStream<'_> {}

impl<'a> Stream for UploadStream<'a> {
    type Item = Chunk;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.left == Some(0) {
            return Poll::Ready(None);
        }

        let this = self.get_mut();
        let underlying = Pin::new(&mut this.stream);
        let res = Stream::poll_next(underlying, cx);
        match &res {
            Poll::Ready(Some(Chunk::Data(data))) => {
                this.left = this.left.map(|x| x.saturating_sub(data.len()));
            }
            Poll::Ready(Some(Chunk::Aborted(..))) => {
                this.left = Some(0);
            }
            _ => {}
        }

        res
    }
}

impl<'a> UploadStream<'a> {
    pub const fn left(&self) -> Option<usize> {
        self.left
    }

    pub const fn unknown_size(stream: BoxedStream<'a>) -> Self {
        Self { stream, left: None }
    }

    pub const fn known_size(stream: BoxedStream<'a>, total_size: usize) -> Self {
        Self {
            stream,
            left: Some(total_size),
        }
    }
}
