use super::keywords;

use syn::{
    parse,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Path, Token,
};

pub struct DemuxArm {
    pub mut_keyword: Option<Token![mut]>,
    pub new_stream: Ident,
    pub variant: Path,
    pub expr: Expr,
}

impl Parse for DemuxArm {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut_keyword = input.parse()?;
        let new_stream = input.parse()?;

        input.parse::<keywords::of>()?;
        let variant = input.parse()?;

        input.parse::<Token![=>]>()?;
        let expr = input.parse::<Expr>()?;

        Ok(Self { mut_keyword, new_stream, variant, expr })
    }
}

pub struct Demux {
    pub stream: Ident,
    pub arms: Punctuated<DemuxArm, Token![,]>,
}

impl Parse for Demux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let stream = input.parse()?;
        input.parse::<Token![->]>()?;
        let arms = Punctuated::parse_terminated(input)?;

        Ok(Self { stream, arms })
    }
}

pub mod gen {
    use crate::demux::{Demux, DemuxArm};
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};

    pub fn channels(count: usize) -> TokenStream {
        let mut expanded = TokenStream::new();

        for i in 0..count {
            let tx = format_ident!("tx_{}", i);
            let rx = format_ident!("rx_{}", i);

            expanded.extend(quote! {
                let (#tx, #rx) = tokio::sync::mpsc::unbounded_channel();
                let #tx = std::sync::Arc::new(#tx);
            });
        }

        expanded
    }

    pub fn dispatch(Demux { stream, arms }: &Demux) -> TokenStream {
        let cloned_senders = cloned_senders(arms.len());
        let dispatcher_arms = dispatcher_arms(arms.iter());

        quote! {
            tokio::spawn(futures::StreamExt::for_each(#stream, move |update| {
                #cloned_senders

                async move {
                    match update {
                        #dispatcher_arms
                    }
                }
            }));
        }
    }

    pub fn join<'a, I>(arms: I) -> TokenStream
    where
        I: Iterator<Item = &'a DemuxArm>,
    {
        let mut expanded = TokenStream::new();

        for (i, DemuxArm { mut_keyword, new_stream, expr, .. }) in arms.enumerate() {
            let rx = format_ident!("rx_{}", i);

            expanded.extend(quote! {
                async move {
                    let #mut_keyword #new_stream = #rx;
                    #expr
                },
            });
        }

        quote! { tokio::join!(#expanded); }
    }

    fn cloned_senders(count: usize) -> TokenStream {
        let mut expanded = TokenStream::new();

        for i in 0..count {
            let tx = format_ident!("tx_{}", i);

            expanded.extend(quote! {
                let #tx = std::sync::Arc::clone(&#tx);
            });
        }

        expanded
    }

    fn dispatcher_arms<'a, I>(arms: I) -> TokenStream
    where
        I: Iterator<Item = &'a DemuxArm>,
    {
        let mut expanded = TokenStream::new();

        for (i, DemuxArm { variant, .. }) in arms.enumerate() {
            let tx = format_ident!("tx_{}", i);

            expanded.extend(quote! {
                #variant (update) => #tx.send(update).expect("RX has been either dropped or closed"),
            });
        }

        expanded
    }
}
