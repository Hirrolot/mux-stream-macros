# mux-stream
[![Continious integration](https://github.com/Hirrolot/mux-stream/workflows/Rust/badge.svg)](https://github.com/Hirrolot/mux-stream/actions)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io/crates/mux-stream)
[![Docs.rs](https://img.shields.io/badge/docs.rs-v0.1.0-blue.svg)](https://docs.rs/mux-stream)

This library provides macros for (de)multiplexing Rusty streams.

 - Multiplexing: `Stream<A>, Stream<B>, Stream<C>` -> `Stream<A | B | C>`
 - Demultiplexing: `Stream<A | B | C>` -> `Stream<A>, Stream<B>, Stream<C>`

## Demultiplexing

You can think of demultiplexing as of `match` on streams:

```rust
use mux_stream::ui.demux;

use futures::StreamExt;
use tokio::stream;

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

ui.demux!(
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
)(stream.boxed()).await;
```

## Multiplexing

Multiplexing gathers updates from several streams into one in an orderless manner:

```rust
use mux_stream::mux;

use std::{collections::HashSet, iter::FromIterator};

use futures::StreamExt;
use tokio::{stream, sync::mpsc::UnboundedReceiver};

#[derive(Debug)]
enum MyEnum {
    A(i32),
    B(u8),
    C(&'static str),
}


let i32_values = HashSet::from_iter(vec![123, 811]);
let u8_values = HashSet::from_iter(vec![88]);
let str_values = HashSet::from_iter(vec!["Hello", "ABC"]);

let result: UnboundedReceiver<MyEnum> = mux! {
    stream::iter(i32_values.clone()) of MyEnum::A,
    stream::iter(u8_values.clone()) of MyEnum::B,
    stream::iter(str_values.clone()) of MyEnum::C
};

let (i32_results, u8_results, str_results) = result
    .fold(
        (HashSet::new(), HashSet::new(), HashSet::new()),
        |(mut i32_results, mut u8_results, mut str_results), update| async move {
            match update {
                MyEnum::A(x) => i32_results.insert(x),
                MyEnum::B(x) => u8_results.insert(x),
                MyEnum::C(x) => str_results.insert(x),
            };

            (i32_results, u8_results, str_results)
        },
    )
    .await;

assert_eq!(i32_results, i32_values);
assert_eq!(u8_results, u8_values);
assert_eq!(str_results, str_values);
```
