use super::common::ConcatTokenStreams;

use proc_macro2::TokenStream;
use quote::quote;
use std::iter;
use syn::{
    braced,
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Ident, Token,
};

#[macro_use]
macro_rules! input_stream {
    ($i:expr) => {
        quote::format_ident!("input_stream_{}", $i)
    };
}

type VariantName = Ident;

struct Mux {
    enum_name: Ident,
    #[allow(dead_code)]
    brace_token: token::Brace,
    variants: Punctuated<VariantName, Token![,]>,
}

impl Parse for Mux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let content;
        let enum_name = input.parse()?;
        let brace_token = braced!(content in input);
        let variants = Punctuated::parse_terminated(&content)?;

        if variants.is_empty() {
            return Err(input.error("At least one variant is required"));
        }

        Ok(Self { enum_name, brace_token, variants })
    }
}

pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mux = parse_macro_input!(input as Mux);

    let input_streams = input_streams_params(mux.variants.len());
    let channels = channels(mux.variants.len());
    let dispatch = dispatch(&mux);

    let expanded = quote! {
        (|#input_streams error_handler: Box<dyn Fn(tokio::sync::mpsc::error::SendError<_>)
            -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static>| {
            let error_handler = std::sync::Arc::new(error_handler);
            #channels
            #dispatch
            rx
        })
    };
    expanded.into()
}

fn input_streams_params(arms_count: usize) -> TokenStream {
    (0..arms_count)
        .map(|i| {
            let input_stream = input_stream!(i);

            quote! {
                #input_stream: futures::stream::BoxStream<'static, _>,
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

fn dispatch(mux: &Mux) -> TokenStream {
    let redirections = redirections(&mux);

    quote! {
        tokio::spawn(async move {
            tokio::join!(#redirections);
        });
    }
}

fn redirections(Mux { enum_name, variants, .. }: &Mux) -> TokenStream {
    variants
        .iter()
        .enumerate()
        .map(|(i, destination_variant)| {
            let tx = tx!(i);
            let input_stream = input_stream!(i);
            let destination_variant = quote! { #enum_name::#destination_variant };

            quote! {{
                let error_handler = std::sync::Arc::clone(&error_handler);

                async move {
                    futures::StreamExt::for_each(#input_stream, move |update| {
                        let #tx = std::sync::Arc::clone(&#tx);
                        let error_handler = std::sync::Arc::clone(&error_handler);

                        async move {
                            if let Err(error) = #tx.send(#destination_variant(update)) {
                                error_handler(error).await;
                            }
                        }
                    }).await;
                }
            },}
        })
        .concat_token_streams()
}
