use mux_stream::demux;

use tokio::stream;

enum MyEnum {
    A(i32),
}

fn main() {
    let _ = async move {
        let stream = stream::iter::<()>(vec![]);
        demux! {
            stream ->
                i32_stream of MyEnum::A => i32_stream.next().await
        }
    };
}
