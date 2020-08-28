use proc_macro2::TokenStream;

pub trait ConcatTokenStreams {
    fn concat_token_streams(self) -> TokenStream;
}

impl<I> ConcatTokenStreams for I
where
    I: Iterator<Item = TokenStream>,
{
    fn concat_token_streams(self) -> TokenStream {
        let mut result = TokenStream::new();
        result.extend(self);
        result
    }
}
