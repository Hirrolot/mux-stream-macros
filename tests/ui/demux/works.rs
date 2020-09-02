use mux_stream_macros::demux_with_error_handler;

use futures::{FutureExt, StreamExt};
use tokio::stream;

#[derive(Debug)]
enum MyEnum {
    A(i32),
    B(f64),
    C(&'static str),
}

#[tokio::main]
async fn main() {
    let stream = stream::iter(vec![
        MyEnum::A(123),
        MyEnum::B(24.241),
        MyEnum::C("Hello"),
        MyEnum::C("ABC"),
        MyEnum::A(811),
    ]);

    let (mut i32_stream, mut f64_stream, mut str_stream) =
        demux_with_error_handler!(MyEnum::A, MyEnum::B, MyEnum::C)
            (Box::new(|error| async move { panic!("{}", error); }.boxed()))
            (stream.boxed());

    assert_eq!(i32_stream.next().await, Some(123));
    assert_eq!(i32_stream.next().await, Some(811));
    assert_eq!(i32_stream.next().await, None);

    assert_eq!(f64_stream.next().await, Some(24.241));
    assert_eq!(f64_stream.next().await, None);

    assert_eq!(str_stream.next().await, Some("Hello"));
    assert_eq!(str_stream.next().await, Some("ABC"));
    assert_eq!(str_stream.next().await, None);
}
