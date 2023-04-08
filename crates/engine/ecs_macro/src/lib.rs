
use proc_macro::TokenStream;

#[proc_macro_derive(System)]
pub fn derive_operator_system(input: TokenStream) -> TokenStream {
    return TokenStream::new();
}