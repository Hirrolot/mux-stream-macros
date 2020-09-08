use super::ConcatTokenStreams;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse,
    parse::{Parse, ParseBuffer, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Path, Token,
};

type VariantPath = Path;

struct Demux {
    pub variants: Vec<VariantPath>,
    pub rest: Option<Token![..]>,
}

impl Parse for Demux {
    // TODO: refactor this function.
    fn parse(input: ParseStream) -> parse::Result<Self> {
        #[derive(Clone)]
        enum InputItem {
            Path(VariantPath),
            PathAndDot2(VariantPath, Token![..]),
            Dot2(Token![..]),
        }

        impl Parse for InputItem {
            fn parse(input: &ParseBuffer) -> parse::Result<Self> {
                match input.parse() {
                    Ok(path) => match input.parse() {
                        Ok(dot2) => Ok(Self::PathAndDot2(path, dot2)),
                        Err(_) => Ok(Self::Path(path)),
                    },
                    Err(_) => Ok(Self::Dot2(input.parse()?)),
                }
            }
        }

        let items: Punctuated<InputItem, Token![,]> = Punctuated::parse_terminated(input)?;
        let trailing_comma_presented = items.trailing_punct();
        let items = items.into_iter().collect::<Vec<InputItem>>();

        let last = if items.is_empty() {
            return Err(input.error("At least one variant is required"));
        } else {
            let last = items[items.len() - 1].clone();

            if matches!(last, InputItem::PathAndDot2(_, _) | InputItem::Dot2(_))
                && trailing_comma_presented
            {
                return Err(input.error("A comma after .. is forbidden"));
            }

            last
        };

        let mut variants = Vec::new();
        let items_len = items.len() - 1;
        for item in items.into_iter().take(items_len) {
            match item {
                InputItem::Path(path) => variants.push(path),
                _ => return Err(input.error(".. must be at the end")),
            }
        }

        match last {
            InputItem::Path(path) => {
                variants.push(path);
                Ok(Self { variants, rest: None })
            }
            InputItem::PathAndDot2(path, dot2) => {
                variants.push(path);
                Ok(Self { variants, rest: Some(dot2) })
            }
            InputItem::Dot2(dot2) => Ok(Self { variants, rest: Some(dot2) }),
        }
    }
}

pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let demux = parse_macro_input!(input as Demux);

    let channels = channels(demux.variants.len());
    let dispatch = dispatch(&demux);
    let output_streams = output_streams(demux.variants.len());

    // The formal arguments are boxed owing to the weak type deduction:
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=443698f46d4e1e4ef313b6bb200149d4.
    let expanded = quote! {
        (|error_handler: Box<dyn Fn(tokio::sync::mpsc::error::SendError<_>) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static>| {
            |input_stream: futures::stream::BoxStream<'static, _>| {
                let error_handler = std::sync::Arc::new(error_handler);
                #channels
                #dispatch
                #output_streams
            }
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

    let rest = match demux.rest {
        Some(_) => {
            quote! { _ => {}, }
        }
        None => {
            quote! {}
        }
    };

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
