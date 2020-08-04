use super::keywords;

use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Path, Token,
};

pub struct MuxInputStream {
    pub stream: Expr,
    pub destination_variant: Path,
}

impl Parse for MuxInputStream {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let stream = input.parse()?;
        input.parse::<keywords::of>()?;
        let destination_variant = input.parse()?;

        Ok(Self { stream, destination_variant })
    }
}

pub struct Mux {
    pub input_streams: Punctuated<MuxInputStream, Token![,]>,
}

impl Parse for Mux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let input_streams = Punctuated::parse_terminated(input)?;
        Ok(Self { input_streams })
    }
}

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mux = parse_macro_input!(input as Mux);

    if mux.input_streams.is_empty() {
        let expected = quote! { compile_error!("At least one input stream is required") };
        return expected.into();
    }

    let moved_input_streams = move_input_streams(mux.input_streams.iter());
    let channels = channels(mux.input_streams.len());
    let dispatch = dispatch(&mux);

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

fn move_input_streams<'a, I>(arms: I) -> TokenStream
where
    I: Iterator<Item = &'a MuxInputStream>,
{
    let mut expanded = TokenStream::new();

    for (i, MuxInputStream { stream, .. }) in arms.enumerate() {
        let input_stream = crate::ith_ident("input_stream", i);

        expanded.extend(quote! {
            let #input_stream = #stream;
        });
    }

    expanded
}

fn channels(count: usize) -> TokenStream {
    let mut expanded = quote! {
        let (tx_0, rx) = tokio::sync::mpsc::unbounded_channel();
        let tx_0 = std::sync::Arc::new(tx_0);
    };

    for i in 1..count {
        let tx = crate::ith_ident("tx", i);

        expanded.extend(quote! {
            let #tx = std::sync::Arc::new(std::clone::Clone::clone(&tx_0));
        });
    }

    expanded
}

fn dispatch(Mux { input_streams }: &Mux) -> TokenStream {
    let redirections = redirections(input_streams.iter());

    quote! {
        tokio::spawn(async move {
            tokio::join!(#redirections);
        });
    }
}

fn redirections<'a, I>(arms: I) -> TokenStream
where
    I: Iterator<Item = &'a MuxInputStream>,
{
    let mut expanded = TokenStream::new();

    for (i, MuxInputStream { destination_variant, .. }) in arms.enumerate() {
        let tx = crate::ith_ident("tx", i);
        let input_stream = crate::ith_ident("input_stream", i);

        expanded.extend(quote! {
                async move {
                    futures::StreamExt::for_each(#input_stream, move |update| {
                        let #tx = std::sync::Arc::clone(&#tx);

                        async move {
                            #tx.send(#destination_variant(update)).expect("RX has been either dropped or closed");
                        }
                    }).await;
                },
            });
    }

    expanded
}
