use mux_stream::*;

#[cfg(test)]
mod tests {
    use super::*;

    use futures::StreamExt;
    use tokio::stream;

    #[derive(Debug)]
    enum MyEnum {
        A(i32),
        B(f64),
        C(&'static str),
    }

    #[tokio::test]
    async fn demux_works() {
        let stream = stream::iter(vec![
            MyEnum::A(123),
            MyEnum::B(24.241),
            MyEnum::C("Hello"),
            MyEnum::C("ABC"),
            MyEnum::A(811),
        ]);

        demux! {
            stream ->
                i32_stream of MyEnum::A =>
                    assert_eq!(i32_stream.collect::<Vec<i32>>().await, vec![123, 811]),
                f64_stream of MyEnum::B =>
                    assert_eq!(f64_stream.collect::<Vec<f64>>().await, vec![24.241]),
                str_stream of MyEnum::C =>
                    assert_eq!(str_stream.collect::<Vec<&'static str>>().await, vec!["Hello", "ABC"]),
        };
    }

    #[test]
    fn demux_panics_on_empty_arms() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/demux/panics_on_empty_arms.rs");
    }
}
