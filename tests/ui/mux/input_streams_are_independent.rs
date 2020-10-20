// Shows that no input stream can slow down the output stream, for example, if
// it accidentally stops consuming items, if at least one another input stream
// is active.

use mux_stream_macros::mux;

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{FutureExt, Stream, StreamExt};
use tokio::{stream, sync::mpsc::UnboundedReceiver};

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
    let mut result: UnboundedReceiver<MyEnum> =
        mux!(MyEnum::A, MyEnum::B, MyEnum::C)(
            InfStream.boxed(),
            stream::iter(vec![88, 25, 66, 11, 6, 0, 90]).boxed(),
            stream::iter(vec!["Hello", "ABC", "bla-bla-bla", "badam"]).boxed(),
            Box::new(|error| {
                async move {
                    panic!("{}", error);
                }.boxed()
            })
        );

    // Be sure that the last two streams are completely processed.
    for _ in 0..(7 + 4) {
        assert!(result.next().await.is_some());
    }
}
