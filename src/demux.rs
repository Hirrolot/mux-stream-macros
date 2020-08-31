use super::{keywords, ConcatTokenStreams};

use syn::{
    parse,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, Path, Token,
};

struct DemuxArm {
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

struct Demux {
    pub arms: Punctuated<DemuxArm, Token![,]>,
}

impl Parse for Demux {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let arms = Punctuated::parse_terminated(input)?;

        Ok(Self { arms })
    }
}

use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote! {
        mux_stream::demux_with_error_handler!(#input)(
            Box::new(|_error| futures::future::FutureExt::boxed(async { }))
        )
    };
    expanded.into()
}

pub fn gen_panicking(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote! {
        mux_stream::demux_with_error_handler!(#input)(
            Box::new(|_error| futures::future::FutureExt::boxed(async {
                panic!("RX has been either dropped or closed");
            }))
        )
    };
    expanded.into()
}

pub fn gen_with_error_handler(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let demux = parse_macro_input!(input as Demux);

    if demux.arms.is_empty() {
        let expected = quote! { compile_error!("At least one arm is required") };
        return expected.into();
    }

    let channels = channels(demux.arms.len());
    let dispatch = dispatch(&demux);
    let join = join(demux.arms.iter());

    // The formal arguments are boxed owing to the weak type deduction:
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=443698f46d4e1e4ef313b6bb200149d4.
    let expanded = quote! {
        (|error_handler: Box<dyn Fn(tokio::sync::mpsc::error::SendError<_>) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static>| {
            |input_stream: futures::stream::BoxStream<'static, _>| async move {
                let error_handler = std::sync::Arc::new(error_handler);
                #channels
                #dispatch
                #join
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

fn dispatch(Demux { arms, .. }: &Demux) -> TokenStream {
    let cloned_senders = cloned_senders(arms.len());
    let dispatcher_arms = dispatcher_arms(arms.iter());

    quote! {
        tokio::spawn(futures::StreamExt::for_each(input_stream, move |update| {
            #cloned_senders
            let error_handler = std::sync::Arc::clone(&error_handler);

            async move {
                match update {
                    #dispatcher_arms
                }
            }
        }));
    }
}

fn join<'a, I>(arms: I) -> TokenStream
where
    I: Iterator<Item = &'a DemuxArm>,
{
    let expanded = arms
        .enumerate()
        .map(|(i, DemuxArm { mut_keyword, new_stream, expr, .. })| {
            let rx = rx!(i);

            quote! {
                async move {
                    let #mut_keyword #new_stream = #rx;
                    #expr
                },
            }
        })
        .concat_token_streams();

    quote! { tokio::join!(#expanded); }
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

fn dispatcher_arms<'a, I>(arms: I) -> TokenStream
where
    I: Iterator<Item = &'a DemuxArm>,
{
    arms.enumerate()
        .map(|(i, DemuxArm { variant, .. })| {
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
