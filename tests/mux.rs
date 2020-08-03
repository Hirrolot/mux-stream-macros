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

#[tokio::test]
async fn mux_works() {
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
}
