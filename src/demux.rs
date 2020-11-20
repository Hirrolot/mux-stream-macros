use super::common::ConcatTokenStreams;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Path, Token,
};

type VariantPath = Path;

struct Demux {
    pub rest: Option<Token![..]>,
    pub variants: Punctuated<VariantPath, Token![,]>,
}

impl Parse for Demux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let rest = input.parse()?;
        let variants = Punctuated::parse_terminated(input)?;

        if variants.is_empty() {
            return Err(input.error("At least one variant is required"));
        }

        Ok(Self { rest, variants })
    }
}

pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let demux = parse_macro_input!(input as Demux);

    let channels = channels(demux.variants.len());
    let dispatch = dispatch(&demux);
    let output_streams = output_streams(demux.variants.len());

    let expanded = quote! {
        (|input_stream, error_handler: Box<dyn Fn(tokio::sync::mpsc::error::SendError<_>) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static>| {
            let error_handler = std::sync::Arc::new(error_handler);
            #channels
            #dispatch
            #output_streams
        })
    };
    expanded.into()
}

fn channels(count: usize) -> TokenStream {
    (0..count)
        .map(|i| {
            let tx = tx!(i);
            let rx = rx!(i);

            quote! {
                let (#tx, #rx) = tokio::sync::mpsc::unbounded_channel();
                let #tx = std::sync::Arc::new(#tx);
            }
        })
        .concat_token_streams()
}

fn dispatch(demux: &Demux) -> TokenStream {
    let cloned_senders = cloned_senders(demux.variants.len());
    let dispatcher_arms = dispatcher_arms(&demux);

    let rest = demux.rest.map(|_| quote! { _ => {}, });

    quote! {
        tokio::spawn(futures::StreamExt::for_each(input_stream, move |update| {
            #cloned_senders
            let error_handler = std::sync::Arc::clone(&error_handler);

            async move {
                match update {
                    #dispatcher_arms
                    #rest
                }
            }
        }));
    }
}

fn output_streams(count: usize) -> TokenStream {
    let expanded = (0..count)
        .map(|i| {
            let rx = rx!(i);

            quote! {
                #rx,
            }
        })
        .concat_token_streams();

    quote! { (#expanded) }
}

fn cloned_senders(count: usize) -> TokenStream {
    (0..count)
        .map(|i| {
            let tx = tx!(i);

            quote! {
                let #tx = std::sync::Arc::clone(&#tx);
            }
        })
        .concat_token_streams()
}

fn dispatcher_arms(Demux { variants, .. }: &Demux) -> TokenStream {
    variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let tx = tx!(i);

            quote! {
                #variant (update) => if let Err(error) = #tx.send(update) {
                    let tokio::sync::mpsc::error::SendError(value) = error;
                    let error = tokio::sync::mpsc::error::SendError(#variant (value));
                    error_handler(error).await;
                },
            }
        })
        .concat_token_streams()
}
