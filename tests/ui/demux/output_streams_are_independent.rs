// Shows that no output stream can slow down another one, for example, if it
// accidentally stops consuming items.

use mux_stream_macros::demux;

use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug)]
enum MyEnum {
    A(i32),
    B(f64),
    C(&'static str),
}

#[tokio::main]
async fn main() {
    let stream = tokio_stream::iter(vec![
        MyEnum::A(123),
        MyEnum::B(24.241),
        MyEnum::C("Hello"),
        MyEnum::C("ABC"),
        MyEnum::A(811),
    ]);

    // We don't touch _i32_stream_skipped, but nonetheless, other streams work as
    // expected.
    let (_i32_stream_skipped, f64_stream, str_stream) =
        demux!(MyEnum { A, B, C })(stream.boxed(), Box::new(|error| {
            Box::pin(async move {
                panic!("{}", error);
            })
        }));

    let mut f64_stream = UnboundedReceiverStream::new(f64_stream);
    let mut str_stream = UnboundedReceiverStream::new(str_stream);

    assert_eq!(f64_stream.next().await, Some(24.241));
    assert_eq!(f64_stream.next().await, None);

    assert_eq!(str_stream.next().await, Some("Hello"));
    assert_eq!(str_stream.next().await, Some("ABC"));
    assert_eq!(str_stream.next().await, None);
}
