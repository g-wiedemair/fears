use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path, parse_macro_input, parse_quote, spanned::Spanned};

pub fn derive_resource(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let feap_ecs_path: Path = crate::feap_ecs_path();

    // We want to raise a compile time error when the generic lifetimes
    // are not bound to 'static lifetime
    let non_static_lifetime_error = ast
        .generics
        .lifetimes()
        .filter(|lifetime| !lifetime.bounds.iter().any(|bound| bound.ident == "static"))
        .map(|param| syn::Error::new(param.span(), "Lifetimes must be 'static"))
        .reduce(|mut err_acc, err| {
            err_acc.combine(err);
            err_acc
        });
    if let Some(err) = non_static_lifetime_error {
        return err.into_compile_error().into();
    }

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #feap_ecs_path::resource::Resource for #struct_name #type_generics #where_clause {
        }
    })
}
