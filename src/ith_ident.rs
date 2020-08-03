use proc_macro2::Ident;
use quote::format_ident;

pub fn ith_ident<Id>(ident: Id, i: usize) -> Ident
where
    Id: AsRef<str>,
{
    format_ident!("{}_{}", ident.as_ref(), i)
}
