use mux_stream_macros::demux;

use futures::{StreamExt};
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

    let (mut i32_stream, mut str_stream) =
        demux!(MyEnum { A, /* B, */ C} ..)
            (stream, Box::new(|error| Box::pin(async move { panic!("{}", error); })));

    assert_eq!(i32_stream.next().await, Some(123));
    assert_eq!(i32_stream.next().await, Some(811));
    assert_eq!(i32_stream.next().await, None);

    assert_eq!(str_stream.next().await, Some("Hello"));
    assert_eq!(str_stream.next().await, Some("ABC"));
    assert_eq!(str_stream.next().await, None);
}
