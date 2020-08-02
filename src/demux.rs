use syn::{
    parse,
    parse::{Parse, ParseStream},
    Expr, Ident, Path, Token,
};

mod keywords {
    syn::custom_keyword!(of);
}

pub struct DemuxArm {
    pub new_stream: Ident,
    pub variant: Path,
    pub expr: Expr,
}

impl Parse for DemuxArm {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let new_stream = input.parse()?;

        input.parse::<keywords::of>()?;
        let variant = input.parse()?;

        input.parse::<Token![=>]>()?;
        let expr = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;

        Ok(Self { new_stream, variant, expr })
    }
}

pub struct Demux {
    pub stream: Ident,
    pub arms: Vec<DemuxArm>,
}

impl Parse for Demux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let stream = input.parse()?;
        input.parse::<Token![->]>()?;

        let mut arms = Vec::new();

        while !input.is_empty() {
            arms.push(input.parse()?);
        }

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
        let dispatcher_arms = dispatcher_arms(&arms);

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

    pub fn join(arms: &[DemuxArm]) -> TokenStream {
        let mut expanded = TokenStream::new();

        for (i, DemuxArm { new_stream, expr, .. }) in arms.iter().enumerate() {
            let rx = format_ident!("rx_{}", i);

            expanded.extend(quote! {
                async move {
                    let #new_stream = #rx;
                    #expr
                },
            });
        }

        quote! { tokio::join!(#expanded) }
    }

    pub fn cloned_senders(count: usize) -> TokenStream {
        let mut expanded = TokenStream::new();

        for i in 0..count {
            let tx = format_ident!("tx_{}", i);

            expanded.extend(quote! {
                let #tx = std::sync::Arc::clone(&#tx);
            });
        }

        expanded
    }

    pub fn dispatcher_arms(arms: &[DemuxArm]) -> TokenStream {
        let mut expanded = TokenStream::new();

        for (i, DemuxArm { variant, .. }) in arms.iter().enumerate() {
            let tx = format_ident!("tx_{}", i);

            expanded.extend(quote! {
                #variant (update) => #tx.send(update).expect("RX has been either dropped or closed"),
            });
        }

        expanded
    }
}
