use super::ConcatTokenStreams;

use proc_macro2::TokenStream;
use quote::quote;
use std::iter;
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Path, Token,
};

pub struct MuxArm {
    pub destination_variant: Path,
}

impl Parse for MuxArm {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let destination_variant = input.parse()?;

        Ok(Self { destination_variant })
    }
}

pub struct Mux {
    pub arms: Punctuated<MuxArm, Token![,]>,
}

impl Parse for Mux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let arms = Punctuated::parse_terminated(input)?;
        Ok(Self { arms })
    }
}

pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mux = parse_macro_input!(input as Mux);

    if mux.arms.is_empty() {
        let expected = quote! { compile_error!("At least one variant is required") };
        return expected.into();
    }

    let input_streams = input_streams(mux.arms.len());
    let channels = channels(mux.arms.len());
    let dispatch = dispatch(&mux);

    let expanded = quote! {
        (|#input_streams| {
            #channels
            #dispatch
            rx
        })
    };
    expanded.into()
}

fn input_streams(arms_count: usize) -> TokenStream {
    (0..arms_count)
        .map(|i| {
            let input_stream = input_stream!(i);

            quote! {
                #input_stream,
            }
        })
        .concat_token_streams()
}

fn channels(count: usize) -> TokenStream {
    iter::once(quote! {
        let (tx_0, rx) = tokio::sync::mpsc::unbounded_channel();
        let tx_0 = std::sync::Arc::new(tx_0);
    })
    .chain((1..count).map(|i| {
        let tx = tx!(i);

        quote! {
            let #tx = std::sync::Arc::new(std::clone::Clone::clone(&tx_0));
        }
    }))
    .concat_token_streams()
}

fn dispatch(Mux { arms: input_streams }: &Mux) -> TokenStream {
    let redirections = redirections(input_streams.iter());

    quote! {
        tokio::spawn(async move {
            tokio::join!(#redirections);
        });
    }
}

fn redirections<'a, I>(arms: I) -> TokenStream
where
    I: Iterator<Item = &'a MuxArm>,
{
    arms.enumerate().map(|(i, MuxArm { destination_variant, .. })| {
        let tx = tx!(i);
        let input_stream = input_stream!(i);

        quote! {
            async move {
                futures::StreamExt::for_each(#input_stream, move |update| {
                    let #tx = std::sync::Arc::clone(&#tx);

                    async move {
                        #tx.send(#destination_variant(update)).expect("RX has been either dropped or closed");
                    }
                }).await;
            },
        }
    }).concat_token_streams()
}
