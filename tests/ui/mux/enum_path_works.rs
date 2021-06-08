use mux_stream_macros::mux;

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
        tokio_stream::iter(vec![123, 811]),
        tokio_stream::iter(vec![88f64]),
        tokio_stream::iter(vec!["Hello", "ABC"]),
        Box::new(|_error| Box::pin(async move { }))
    );
}
