//! This library provides macros for (de)multiplexing Rusty streams.
//!
//! See [our GitHub repository](https://github.com/Hirrolot/mux-stream) for a high-level overview.

#![deny(unsafe_code)]

#[macro_use]
mod common;
mod demux;
mod mux;

use common::ConcatTokenStreams;

use proc_macro::TokenStream;

/// Multiplexes several streams into one.
///
/// Accepts a non-empty list of paths to variants of an enumeration, possibly
/// with a trailing comma. All enumeration variants shall be defined as variants
/// taking a single unnamed parameter.
///
/// Expands to a closure that has the same number of formal arguments as the
/// number of paths specified; each one must implement [`Stream<T>`], where `T`
/// is a type of a single unnamed parameter of the corresponding variant. This
/// closure returns [`tokio::sync::mpsc::UnboundedReceiver`] of your enumeration
/// type.
///
/// It propagates updates into the result stream in any order, simultaneously
/// from all the provided input streams (in a separate [Tokio task]).
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
/// let result: UnboundedReceiver<MyEnum> = mux!(MyEnum::A, MyEnum::B, MyEnum::C)(
///     stream::iter(i32_values.clone()),
///     stream::iter(u8_values.clone()),
///     stream::iter(str_values.clone()),
/// );
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
/// Hash sets are used here owing to the obvious absence of order preservation
/// of updates from input streams.
///
/// [`Stream<T>`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
/// [`tokio::sync::mpsc::UnboundedReceiver`]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html
/// [Tokio task]: https://docs.rs/tokio/latest/tokio/task/index.html
#[proc_macro]
pub fn mux(input: TokenStream) -> TokenStream {
    mux::gen(input)
}

/// Demultiplexer with a silent error handler.
///
/// Expands to `demux_with_error_handler!(...)(/* A no-op closure */)`.
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
/// let (mut i32_stream, mut f64_stream, mut str_stream) =
///     demux!(MyEnum::A, MyEnum::B, MyEnum::C)(stream.boxed());
///
/// assert_eq!(i32_stream.next().await, Some(123));
/// assert_eq!(i32_stream.next().await, Some(811));
/// assert_eq!(i32_stream.next().await, None);
///
/// assert_eq!(f64_stream.next().await, Some(24.241));
/// assert_eq!(f64_stream.next().await, None);
///
/// assert_eq!(str_stream.next().await, Some("Hello"));
/// assert_eq!(str_stream.next().await, Some("ABC"));
/// assert_eq!(str_stream.next().await, None);
/// # }
/// ```
#[proc_macro]
pub fn demux(input: TokenStream) -> TokenStream {
    demux::gen(input)
}

/// Demultiplexer with a panicking error handler.
///
/// Expands to `demux_with_error_handler!(...)(/* A panicking closure */)`.
///
/// # Example
///
/// ```
/// use mux_stream::demux_panicking;
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
/// let (mut i32_stream, mut f64_stream, mut str_stream) =
///     demux_panicking!(MyEnum::A, MyEnum::B, MyEnum::C)(stream.boxed());
///
/// assert_eq!(i32_stream.next().await, Some(123));
/// assert_eq!(i32_stream.next().await, Some(811));
/// assert_eq!(i32_stream.next().await, None);
///
/// assert_eq!(f64_stream.next().await, Some(24.241));
/// assert_eq!(f64_stream.next().await, None);
///
/// assert_eq!(str_stream.next().await, Some("Hello"));
/// assert_eq!(str_stream.next().await, Some("ABC"));
/// assert_eq!(str_stream.next().await, None);
/// # }
/// ```
#[proc_macro]
pub fn demux_panicking(input: TokenStream) -> TokenStream {
    demux::gen_panicking(input)
}

/// Demultiplexes a stream into several others with a custom error handler.
///
/// Accepts a non-empty list of paths to variants of an enumeration, possibly
/// with a trailing comma. All enumeration variants shall be defined as variants
/// taking a single unnamed parameter.
///
/// Expands to:
///
/// ```ignore
/// |error_handler: Box<dyn Fn(tokio::sync::mpsc::error::SendError<_>) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static>| {
///     |input_stream: futures::stream::BoxStream<'static, _>| { /* ... */ }
/// }
/// ```
///
/// Thus, the returned closure is [curried]. After applying two arguments to it
/// (`(...)(...)`), you obtain a future of type
/// `(tokio::sync::mpsc::UnboundedReceiver<T[1]>, ...,
/// tokio::sync::mpsc::UnboundedReceiver<T[n]>)`, where `T[i]` is a type of a
/// single unnamed parameter of the corresponding provided variant.
///
/// `input_stream` is a stream of your enumeration to be demiltiplexed. Each
/// coming update from `input_stream` will be pushed into the corresponding
/// output stream immediately, in a separate [Tokio task].
///
/// `error_handler` is invoked when a demultiplexer fails to send an update
/// from `input_stream` into one of receivers.
///
/// # Example
/// ```
/// use mux_stream::demux_with_error_handler;
///
/// use futures::{future::FutureExt, StreamExt};
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
/// let (mut i32_stream, mut f64_stream, mut str_stream) =
///     demux_with_error_handler!(MyEnum::A, MyEnum::B, MyEnum::C)(Box::new(|error| {
///         async move {
///             dbg!(error);
///         }
///         .boxed()
///     }))(stream.boxed());
///
/// assert_eq!(i32_stream.next().await, Some(123));
/// assert_eq!(i32_stream.next().await, Some(811));
/// assert_eq!(i32_stream.next().await, None);
///
/// assert_eq!(f64_stream.next().await, Some(24.241));
/// assert_eq!(f64_stream.next().await, None);
///
/// assert_eq!(str_stream.next().await, Some("Hello"));
/// assert_eq!(str_stream.next().await, Some("ABC"));
/// assert_eq!(str_stream.next().await, None);
/// # }
/// ```
///
/// [curried]: https://en.wikipedia.org/wiki/Currying
/// [Tokio task]: https://docs.rs/tokio/latest/tokio/task/index.html
#[proc_macro]
pub fn demux_with_error_handler(input: TokenStream) -> TokenStream {
    demux::gen_with_error_handler(input)
}
