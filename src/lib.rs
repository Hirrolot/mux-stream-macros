#![deny(unsafe_code)]

mod demux;

use demux::{gen, Demux};

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Demultiplexes a stream into several others.
///
/// ```
/// use futures::StreamExt;
/// use mux_stream::demux;
/// use tokio::stream;
///
/// #[derive(Debug)]
/// enum MyEnum {
///     A(i32),
///     B(f64),
///     C(&'static str),
/// }
///
/// # #[tokio::main]
/// # async fn main_() {
/// let stream = stream::iter(vec![
///     MyEnum::A(123),
///     MyEnum::B(24.241),
///     MyEnum::C("Hello"),
///     MyEnum::C("ABC"),
///     MyEnum::A(811),
/// ]);
///
/// demux! {
///     stream of
///         i32_stream of MyEnum::A =>
///             assert_eq!(i32_stream.collect::<Vec<i32>>().await, vec![123, 811]),
///         f64_stream of MyEnum::B =>
///             assert_eq!(f64_stream.collect::<Vec<f64>>().await, vec![24.241]),
///         str_stream of MyEnum::C =>
///             assert_eq!(str_stream.collect::<Vec<&'static str>>().await, vec!["Hello", "ABC"]),
/// };
/// # }
/// ```
#[proc_macro]
pub fn demux(input: TokenStream) -> TokenStream {
    let demux = parse_macro_input!(input as Demux);

    if demux.arms.is_empty() {
        let expected = quote! { compile_error!("At least one arm is required") };
        return expected.into();
    }

    let channels = gen::channels(demux.arms.len());
    let dispatch = gen::dispatch(&demux);
    let join = gen::join(&demux.arms);

    let expanded = quote! {
        #channels
        #dispatch
        #join
    };
    expanded.into()
}
