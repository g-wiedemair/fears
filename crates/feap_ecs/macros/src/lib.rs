extern crate proc_macro;

use feap_macro_utils::{FeapManifest, derive_label};
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{DeriveInput, parse_macro_input};

pub(crate) fn feap_ecs_path() -> syn::Path {
    FeapManifest::shared(|manifest| manifest.get_path("feap_ecs"))
}

/// Derive macro generating an impl of the trait `ScheduleLabel`.
///
/// This does not work for unions.
#[proc_macro_derive(ScheduleLabel)]
pub fn derive_schedule_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut trait_path = feap_ecs_path();
    trait_path.segments.push(format_ident!("schedule").into());
    trait_path
        .segments
        .push(format_ident!("ScheduleLabel").into());
    derive_label(input, "ScheduleLabel", &trait_path)
}
