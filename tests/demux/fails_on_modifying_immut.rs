use mux_stream::demux;

enum MyEnum {
    A(i32),
}

fn main() {
    demux!(i32_stream of MyEnum::A => i32_stream.next().await);
}
