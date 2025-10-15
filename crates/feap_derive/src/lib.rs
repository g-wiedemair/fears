#![forbid(unsafe_code)]

extern crate proc_macro;

use feap_macro_utils::{FeapManifest, derive_label};
use proc_macro::TokenStream;
use quote::format_ident;

/// Generates an impl of the `AppLabel` trait
/// This does not work for unions
#[proc_macro_derive(AppLabel)]
pub fn derive_app_label(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let mut trait_path = FeapManifest::shared(|manifest| manifest.get_path("feap_app"));
    trait_path.segments.push(format_ident!("AppLabel").into());
    derive_label(input, "AppLabel", &trait_path)
}
