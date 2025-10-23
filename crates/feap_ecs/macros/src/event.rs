use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, Path, Type};

pub const EVENT: &str = "event";
pub const TRIGGER: &str = "trigger";

pub fn derive_event(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let feap_ecs_path: Path = crate::feap_ecs_path();

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let mut processed_attrs = Vec::new();
    let mut trigger: Option<Type> = None;

    for attr in ast.attrs.iter().filter(|attr| attr.path().is_ident(EVENT)) {
        if let Err(e) = attr.parse_nested_meta(|meta| match meta.path.get_ident() {
            Some(ident) if processed_attrs.iter().any(|i| ident == i) => {
                Err(meta.error(format!("duplicate attribute: {ident}")))
            }
            Some(ident) if ident == TRIGGER => {
                trigger = Some(meta.value()?.parse()?);
                processed_attrs.push(TRIGGER);
                Ok(())
            }
            Some(ident) => Err(meta.error(format!("unsupported attribute: {ident}"))),
            None => Err(meta.error("expected identifier")),
        }) {
            return e.to_compile_error().into();
        }
    }

    let trigger = if let Some(trigger) = trigger {
        quote! {#trigger}
    } else {
        quote! {#feap_ecs_path::event::GlobalTrigger}
    };

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #feap_ecs_path::event::Event for #struct_name #type_generics #where_clause {
            type Trigger<'a> = #trigger;
        }
    })
}
