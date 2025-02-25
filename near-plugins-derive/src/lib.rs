use proc_macro::{self, TokenStream};

mod full_access_key_fallback;
mod ownable;
mod pausable;
mod upgradable;
mod utils;

#[proc_macro_derive(Ownable, attributes(ownable))]
pub fn derive_ownable(input: TokenStream) -> TokenStream {
    ownable::derive_ownable(input)
}

#[proc_macro_attribute]
pub fn only(attrs: TokenStream, item: TokenStream) -> TokenStream {
    ownable::only(attrs, item)
}

#[proc_macro_derive(Upgradable, attributes(upgradable))]
pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    upgradable::derive_upgradable(input)
}

#[proc_macro_derive(FullAccessKeyFallback)]
pub fn derive_fak_fallback(input: TokenStream) -> TokenStream {
    full_access_key_fallback::derive_fak_fallback(input)
}

#[proc_macro_derive(Pausable)]
pub fn derive_pausable(input: TokenStream) -> TokenStream {
    pausable::derive_pausable(input)
}

#[proc_macro_attribute]
pub fn pause(attrs: TokenStream, item: TokenStream) -> TokenStream {
    pausable::pause(attrs, item)
}

#[proc_macro_attribute]
pub fn if_paused(attrs: TokenStream, item: TokenStream) -> TokenStream {
    pausable::if_paused(attrs, item)
}
