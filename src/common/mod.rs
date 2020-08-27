mod concat_token_streams;
pub mod keywords;

pub use concat_token_streams::ConcatTokenStreams;

#[macro_use]
pub mod private_macros {
    macro_rules! tx {
        ($i:expr) => {
            quote::format_ident!("tx_{}", $i)
        };
    }

    macro_rules! rx {
        ($i:expr) => {
            quote::format_ident!("rx_{}", $i)
        };
    }

    macro_rules! input_stream {
        ($i:expr) => {
            quote::format_ident!("input_stream_{}", $i)
        };
    }
}
