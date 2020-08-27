use mux_stream::demux;

use futures::StreamExt;

enum MyEnum {
    A(i32),
}

fn main() {
    demux!(i32_stream of MyEnum::A => i32_stream.next().await);
}
