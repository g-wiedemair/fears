use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Derive a label trait
///
pub fn derive_label(
    input: syn::DeriveInput,
    trait_name: &str,
    trait_path: &syn::Path,
) -> TokenStream {
    if let syn::Data::Union(_) = &input.data {
        let message = format!("Cannot derive {trait_name} for unions.");
        return quote_spanned! {
            input.span() => compile_error!(#message);
        }
        .into();
    }

    let ident = input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause.predicates.push(
        syn::parse2(quote! {
            Self: 'static + Send + Sync + Clone + Eq + ::core::fmt::Debug + ::core::hash::Hash
        })
        .unwrap(),
    );
    quote! {
        // To ensure alloc is available, but also prevent its name from clashing, we place the implementation inside an anonymous constant
        const _: () = {
            extern crate alloc;

            impl #impl_generics #trait_path for #ident #ty_generics #where_clause {
                fn dyn_clone(&self) -> alloc::boxed::Box<dyn #trait_path> {
                    alloc::boxed::Box::new(::core::clone::Clone::clone(self))
                }
            }
        };
    }
    .into()
}
