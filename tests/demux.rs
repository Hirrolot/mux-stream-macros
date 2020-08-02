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
    }

    #[test]
    fn demux_panics_on_empty_arms() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/demux/panics_on_empty_arms.rs");
    }
}
