use proc_macro2::TokenStream;

pub trait ConcatTokenStreams {
    fn concat_token_streams(self) -> TokenStream;
}

impl<I> ConcatTokenStreams for I
where
    I: Iterator<Item = TokenStream>,
{
    fn concat_token_streams(self) -> TokenStream {
        self.fold(TokenStream::new(), |mut acc, item| {
            acc.extend(item);
            acc
        })
    }
}
