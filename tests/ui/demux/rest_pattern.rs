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

    let (i32_stream, str_stream) =
        demux!(MyEnum { A, /* B, */ C} ..)
            (stream, Box::new(|error| Box::pin(async move { panic!("{}", error); })));

    let mut i32_stream = UnboundedReceiverStream::new(i32_stream);
    let mut str_stream = UnboundedReceiverStream::new(str_stream);

    assert_eq!(i32_stream.next().await, Some(123));
    assert_eq!(i32_stream.next().await, Some(811));
    assert_eq!(i32_stream.next().await, None);

    assert_eq!(str_stream.next().await, Some("Hello"));
    assert_eq!(str_stream.next().await, Some("ABC"));
    assert_eq!(str_stream.next().await, None);
}
