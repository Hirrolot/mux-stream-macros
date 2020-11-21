use mux_stream_macros::demux;

use tokio::stream;

mod abc {
    pub mod def {
        #[derive(Debug)]
        pub enum MyEnum {
            A(i32),
            B(f64),
            C(&'static str),
        }
    }
}

#[tokio::main]
async fn main() {
    let stream = stream::iter(vec![
        abc::def::MyEnum::A(123),
        abc::def::MyEnum::B(24.241),
        abc::def::MyEnum::C("Hello"),
        abc::def::MyEnum::C("ABC"),
        abc::def::MyEnum::A(811),
    ]);

    demux!(abc::def::MyEnum { A, B, C })
        (stream, Box::new(|_error| Box::pin(async move { })));
}
