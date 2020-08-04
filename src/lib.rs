//! This library provides macros for (de)multiplexing Rusty streams.
//!
//!  - Multiplexing: `Stream<A>, Stream<B>, Stream<C>` -> `Stream<A | B | C>`
//!  - Demultiplexing: `Stream<A | B | C>` -> `Stream<A>, Stream<B>, Stream<C>`
//!
//! See [our GitHub repository](https://github.com/Hirrolot/mux-stream) for a high-level overview.

#![deny(unsafe_code)]

mod common;
mod demux;
mod mux;

use common::{ith_ident, keywords};

use proc_macro::TokenStream;

/// Multiplexes several streams into one.
///
/// # Grammar
///
/// ```no_compile
/// input_stream0 of MyEnum::Variant0,
/// ...,
/// input_streamN of MyEnum::VariantN [,]
/// ```
///
/// # Constraints
///
///  - Ith `input_stream` shall implement [`Stream<T>`], where `T` is a type of
///    a single argument of ith `MyEnum::Variant`.
///  - At least one input stream shall be provided.
///  - This macro can be invoked with or without a trailing comma.
///
/// # Semantics
/// Returns [`tokio::sync::mpsc::UnboundedReceiver<MyEnum>`].
///
/// Updates into the result stream may come in any order, simultaneously from
/// all the provided input streams.
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
    mux::gen(input)
}

/// Demultiplexer with a panicking error handler.
///
/// Expands to `demux_with_error_handler!(...)(|error| async move { /*
/// panic!(...) */ })`.
///
/// # Example
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
/// demux!(
///     mut i32_stream of MyEnum::A => {
///         assert_eq!(i32_stream.next().await, Some(123));
///         assert_eq!(i32_stream.next().await, Some(811));
///         assert_eq!(i32_stream.next().await, None);
///     },
///     mut f64_stream of MyEnum::B => {
///         assert_eq!(f64_stream.next().await, Some(24.241));
///         assert_eq!(f64_stream.next().await, None);
///     },
///     mut str_stream of MyEnum::C => {
///         assert_eq!(str_stream.next().await, Some("Hello"));
///         assert_eq!(str_stream.next().await, Some("ABC"));
///         assert_eq!(str_stream.next().await, None);
///     }
/// )(stream).await;
/// # }
/// ```
#[proc_macro]
pub fn demux(input: TokenStream) -> TokenStream {
    demux::gen(input)
}

/// Demultiplexes a stream into several others with a custom error handler.
///
/// # Grammar
///
/// ```no_compile
/// [mut] output_stream0 of MyEnum::Variant0 => expr0,
/// ...
/// [mut] output_streamN of MyEnum::VariantN => exprN [,]
/// ```
///
/// # Contraints
///
///  - Ith `output_stream` is of type
///    [`tokio::sync::mpsc::UnboundedReceiver<T>`], where `T` is a type of a
///    single argument of ith `MyEnum::Variant`.
///  - `MyEnum::Variant0`, ..., `MyEnum::VariantN` shall be defined as variants
///    taking a single unnamed argument.
///  - `expr0`, ..., `exprN` shall be expressions of the same type.
///  - At least one arm shall be provided.
///  - This macro can be invoked with or without a trailing comma.
///
/// # Semantics
/// This macro expands to a closure of a single parameter `error_handler`, which
/// return a closure of a single parameter `input_stream` (implementing
/// [`Stream<MyEnum>`]), which returns a future of a type of `expr0`, ...,
/// `exprN`. Thus, the returned closure is [curried].
///
/// Each coming update from `input_stream` will be pushed into the corresponding
/// output stream immediately.
///
/// `error_handler` is a closure, which returns `F: Future<Output = ()>` and
/// takes [`SendError<MyEnum>`]. It is invoked when a demultiplexer fails to
/// send an update from an input stream into one of receivers.
///
/// # Example
/// ```
/// use mux_stream::demux_with_error_handler;
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
/// demux_with_error_handler!(
///     mut i32_stream of MyEnum::A => {
///         assert_eq!(i32_stream.next().await, Some(123));
///         assert_eq!(i32_stream.next().await, Some(811));
///         assert_eq!(i32_stream.next().await, None);
///     },
///     mut f64_stream of MyEnum::B => {
///         assert_eq!(f64_stream.next().await, Some(24.241));
///         assert_eq!(f64_stream.next().await, None);
///     },
///     mut str_stream of MyEnum::C => {
///         assert_eq!(str_stream.next().await, Some("Hello"));
///         assert_eq!(str_stream.next().await, Some("ABC"));
///         assert_eq!(str_stream.next().await, None);
///     }
/// )(|error| async move {
///     dbg!(error);
/// })(stream).await;
/// # }
/// ```
///
/// [curried]: https://en.wikipedia.org/wiki/Currying
/// [`Stream<MyEnum>`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
/// [`tokio::sync::mpsc::UnboundedReceiver<T>`]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html
/// [`SendError<MyEnum>`]: https://docs.rs/tokio/latest/tokio/sync/mpsc/error/struct.SendError.html
#[proc_macro]
pub fn demux_with_error_handler(input: TokenStream) -> TokenStream {
    demux::gen_with_error_handler(input)
}
