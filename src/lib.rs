//! Internals of [`mux-stream`](https://crates.io/crates/mux-stream). Don't use it directly.

#![deny(unsafe_code)]

#[macro_use]
mod common;
mod demux;
mod mux;

use proc_macro::TokenStream;

#[proc_macro]
pub fn mux(input: TokenStream) -> TokenStream {
    mux::gen(input)
}

#[proc_macro]
pub fn demux(input: TokenStream) -> TokenStream {
    demux::gen(input)
}
