// Shows that no input stream can slow down the output stream, for example, if
// it accidentally stops consuming items, if at least one another input stream
// is active.

use mux_stream_macros::mux;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug)]
enum MyEnum {
    A(i32),
    B(u8),
    C(&'static str),
}

struct InfStream;

impl Stream for InfStream {
    type Item = i32;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

#[tokio::main]
async fn main() {
    let result: UnboundedReceiver<MyEnum> =
        mux!(MyEnum { A, B, C })(
            InfStream,
            tokio_stream::iter(vec![88, 25, 66, 11, 6, 0, 90]),
            tokio_stream::iter(vec!["Hello", "ABC", "bla-bla-bla", "badam"]),
            Box::new(|error| {
                Box::pin(async move {
                    panic!("{}", error);
                })
            })
        );

    let mut result = UnboundedReceiverStream::new(result);

    // Be sure that the last two streams are completely processed.
    for _ in 0..(7 + 4) {
        assert!(result.next().await.is_some());
    }
}
