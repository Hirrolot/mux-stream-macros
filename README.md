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
    stream of
        i32_stream of MyEnum::A => assert_eq!(i32_stream.collect::<Vec<i32>>().await, vec![123, 811]),
        f64_stream of MyEnum::B => assert_eq!(f64_stream.collect::<Vec<f64>>().await, vec![24.241]),
        str_stream of MyEnum::C => assert_eq!(str_stream.collect::<Vec<&'static str>>().await, vec!["Hello", "ABC"]),
};
```
