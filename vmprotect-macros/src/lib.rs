mod protect;
use darling::{Error, FromMeta};
use darling::ast::NestedMeta;
use syn::ItemFn;
use proc_macro::TokenStream;
use protect::ProtectArgs;

#[proc_macro_attribute]
pub fn protected(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => { return TokenStream::from(Error::from(e).write_errors()); }
    };
    let args = match ProtectArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => { return TokenStream::from(e.write_errors()); }
    };
    let input = syn::parse_macro_input!(input as ItemFn);

    let mut context = protect::ProtectContext::new(args, input);

    match context.render() {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
