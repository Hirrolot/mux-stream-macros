//! Internals of [`mux-stream`](https://crates.io/crates/mux-stream). Don't use it directly.

#![deny(unsafe_code)]

#[macro_use]
mod common;
mod demux;
mod mux;

use common::ConcatTokenStreams;

use proc_macro::TokenStream;

#[proc_macro]
pub fn mux(input: TokenStream) -> TokenStream {
    mux::gen(input)
}

#[proc_macro]
pub fn demux_with_error_handler(input: TokenStream) -> TokenStream {
    demux::gen_with_error_handler(input)
}
