//! This library provides macros for (de)multiplexing Rusty streams.
//!
//!  - Multiplexing: `Stream<A>, Stream<B>, Stream<C>` -> `Stream<A | B | C>`
//!  - Demultiplexing: `Stream<A | B | C>` -> `Stream<A>, Stream<B>, Stream<C>`
//!
//! See [our GitHub repository](https://github.com/Hirrolot/mux-stream) for a high-level overview.

#![deny(unsafe_code)]

mod demux;
mod keywords;
mod mux;

use demux::Demux;
use mux::Mux;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Multiplexes several streams into one.
///
/// Grammar:
/// ```
/// mux! {
///     input_stream0 of MyEnum::Variant0,
///     ...,
///     input_streamN of MyEnum::VariantN [,]
/// }
/// ```
///
/// Ith `input_stream` must implement [`Stream<T>`], where `T` is a type of a
/// single argument of ith `MyEnum::Variant`. Returns
/// [`tokio::sync::mpsc::UnboundedReceiver<MyEnum>`].
///
/// Updates into the result stream may come in any order, simultaneously from
/// all the provided input streams.
///
/// This macro can be invoked with or without a trailing comma. At least one
/// input stream must be provided.
///
/// All the provided input streams will be moved.
///
/// ```
/// use mux_stream::mux;
///
/// use std::{collections::HashSet, iter::FromIterator};
///
/// use futures::StreamExt;
/// use tokio::{stream, sync::mpsc::UnboundedReceiver};
///
/// #[derive(Debug)]
/// enum MyEnum {
///     A(i32),
///     B(u8),
///     C(&'static str),
/// }
///
/// # #[tokio::main]
/// # async fn main_() {
///
/// let i32_values = HashSet::from_iter(vec![123, 811]);
/// let u8_values = HashSet::from_iter(vec![88]);
/// let str_values = HashSet::from_iter(vec!["Hello", "ABC"]);
///
/// let result: UnboundedReceiver<MyEnum> = mux! {
///     stream::iter(i32_values.clone()) of MyEnum::A,
///     stream::iter(u8_values.clone()) of MyEnum::B,
///     stream::iter(str_values.clone()) of MyEnum::C
/// };
///
/// let (i32_results, u8_results, str_results) = result
///     .fold(
///         (HashSet::new(), HashSet::new(), HashSet::new()),
///         |(mut i32_results, mut u8_results, mut str_results), update| async move {
///             match update {
///                 MyEnum::A(x) => i32_results.insert(x),
///                 MyEnum::B(x) => u8_results.insert(x),
///                 MyEnum::C(x) => str_results.insert(x),
///             };
///
///             (i32_results, u8_results, str_results)
///         },
///     )
///     .await;
///
/// assert_eq!(i32_results, i32_values);
/// assert_eq!(u8_results, u8_values);
/// assert_eq!(str_results, str_values);
/// # }
/// ```
///
/// [`Stream<T>`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
/// [`tokio::sync::mpsc::UnboundedReceiver<MyEnum>`]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html
#[proc_macro]
pub fn mux(input: TokenStream) -> TokenStream {
    let mux = parse_macro_input!(input as Mux);

    if mux.input_streams.is_empty() {
        let expected = quote! { compile_error!("At least one input stream is required") };
        return expected.into();
    }

    let moved_input_streams = mux::gen::move_input_streams(mux.input_streams.iter());
    let channels = mux::gen::channels(mux.input_streams.len());
    let dispatch = mux::gen::dispatch(&mux);

    let expanded = quote! {
        {
            #moved_input_streams
            #channels
            #dispatch
            rx
        }
    };
    expanded.into()
}

/// Demultiplexes a stream into several others.
///
/// Grammar:
///
/// ```
/// demux! {
///     input_stream ->
///         [mut] output_stream0 of MyEnum::Variant0 => expr,
///         ...
///         [mut] output_streamN of MyEnum::VariantN => expr [,]
/// }
/// ```
///
/// Ith `output_stream` is of type [`tokio::sync::mpsc::UnboundedReceiver<T>`],
/// where `T` is a type of a single argument of ith `MyEnum::Variant`.
/// `input_stream` must implement [`Stream<MyEnum>`].
///
/// Each coming update from `input_stream` will be pushed into the corresponding
/// output stream immediately.
///
/// This macro can be invoked with or without a trailing comma. At least one arm
/// must be provided.
///
/// `input_stream` will be moved.
///
/// ```
/// use mux_stream::demux;
///
/// use futures::StreamExt;
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
///     stream ->
///         mut i32_stream of MyEnum::A => {
///             assert_eq!(i32_stream.next().await, Some(123));
///             assert_eq!(i32_stream.next().await, Some(811));
///             assert_eq!(i32_stream.next().await, None);
///         },
///         mut f64_stream of MyEnum::B => {
///             assert_eq!(f64_stream.next().await, Some(24.241));
///             assert_eq!(f64_stream.next().await, None);
///         },
///         mut str_stream of MyEnum::C => {
///             assert_eq!(str_stream.next().await, Some("Hello"));
///             assert_eq!(str_stream.next().await, Some("ABC"));
///             assert_eq!(str_stream.next().await, None);
///         }
/// }
/// # }
/// ```
///
/// [`Stream<MyEnum>`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
/// [`tokio::sync::mpsc::UnboundedReceiver<T>`]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html
#[proc_macro]
pub fn demux(input: TokenStream) -> TokenStream {
    let demux = parse_macro_input!(input as Demux);

    if demux.arms.is_empty() {
        let expected = quote! { compile_error!("At least one arm is required") };
        return expected.into();
    }

    let channels = demux::gen::channels(demux.arms.len());
    let dispatch = demux::gen::dispatch(&demux);
    let join = demux::gen::join(demux.arms.iter());

    let expanded = quote! {
        {
            #channels
            #dispatch
            #join
        }
    };
    expanded.into()
}
