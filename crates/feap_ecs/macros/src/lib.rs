extern crate proc_macro;
mod component;
mod event;
mod message;

use feap_macro_utils::{derive_label, FeapManifest};
use proc_macro::TokenStream;
use quote::format_ident;
use syn::{parse_macro_input, DeriveInput};

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

/// Derive macro generating an impl of the trait `SystemSet`.
///
/// This does not work for unions.
#[proc_macro_derive(SystemSet)]
pub fn derive_system_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut trait_path = feap_ecs_path();
    trait_path.segments.push(format_ident!("schedule").into());
    trait_path.segments.push(format_ident!("SystemSet").into());
    derive_label(input, "SystemSet", &trait_path)
}

#[proc_macro_derive(
    Component,
    attributes(component, require, relationship, relationship_target, entities)
)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    component::derive_component(input)
}

/// Implement the `Resource` trait.
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    component::derive_resource(input)
}

/// Implement the `Event` trait.
#[proc_macro_derive(Event, attributes(event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    event::derive_event(input)
}

/// Implement the `Message` trait
#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    message::derive_message(input)
}
