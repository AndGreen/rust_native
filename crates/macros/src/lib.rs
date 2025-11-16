mod parser;

use parser::UiRoot;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn ui(tokens: TokenStream) -> TokenStream {
    let root = parse_macro_input!(tokens as UiRoot);
    let expanded = root.expand();
    quote! {{ #expanded }}.into()
}
