```rust
use mux_stream::demux;
use tokio::stream;
use futures::StreamExt;

#[derive(Debug)]
enum MyEnum {
    A(i32),
    B(f64),
    C(&'static str),
}

let stream = stream::iter(vec![
    MyEnum::A(123),
    MyEnum::B(24.241),
    MyEnum::C("Hello"),
    MyEnum::C("ABC"),
    MyEnum::A(811),
]);

demux! {
    stream ->
        mut i32_stream of MyEnum::A => {
            assert_eq!(i32_stream.next().await, Some(123));
            assert_eq!(i32_stream.next().await, Some(811));
            assert_eq!(i32_stream.next().await, None);
        },
        mut f64_stream of MyEnum::B => {
            assert_eq!(f64_stream.next().await, Some(24.241));
            assert_eq!(f64_stream.next().await, None);
        },
        mut str_stream of MyEnum::C => {
            assert_eq!(str_stream.next().await, Some("Hello"));
            assert_eq!(str_stream.next().await, Some("ABC"));
            assert_eq!(str_stream.next().await, None);
        }
};
```
