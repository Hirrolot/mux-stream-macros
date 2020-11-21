use mux_stream_macros::mux;

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
    mux!(abc::def::MyEnum { A, B, C })(
        stream::iter(vec![123, 811]),
        stream::iter(vec![88f64]),
        stream::iter(vec!["Hello", "ABC"]),
        Box::new(|_error| Box::pin(async move { }))
    );
}
