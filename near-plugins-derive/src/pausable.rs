use crate::utils::{cratename, is_near_bindgen_wrapped_or_marshall};
use darling::{FromDeriveInput, FromMeta};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse, parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(pausable), forward_attrs(allow, doc, cfg))]
struct Opts {
    paused_storage_key: Option<String>,
}

pub fn derive_pausable(input: TokenStream) -> TokenStream {
    let cratename = cratename();

    let input = parse_macro_input!(input);
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let DeriveInput { ident, .. } = input;

    let paused_storage_key = opts.paused_storage_key.unwrap_or("__PAUSE__".to_string());

    let output = quote! {
        #[near_bindgen]
        impl Pausable for #ident {
            fn pa_storage_key(&self) -> Vec<u8>{
                (#paused_storage_key).as_bytes().to_vec()
            }

            fn pa_is_paused(&self, key: String) -> bool {
                self.pa_all_paused()
                    .map(|keys| keys.contains(&key) || keys.contains("ALL"))
                    .unwrap_or(false)
            }

            fn pa_all_paused(&self) -> Option<std::collections::HashSet<String>> {
                ::near_sdk::env::storage_read(self.pa_storage_key().as_ref()).map(|value| {
                    std::collections::HashSet::try_from_slice(value.as_ref())
                        .expect("Pausable: Invalid format for paused keys")
                })
            }

            #[#cratename::only(owner)]
            fn pa_pause_feature(&mut self, key: String) {
                let mut paused_keys = self.pa_all_paused().unwrap_or_default();
                paused_keys.insert(key.clone());

                ::near_sdk::log!(#cratename::events::AsEvent::event(
                    &#cratename::pausable::Pause {
                        by: ::near_sdk::env::predecessor_account_id(),
                        key,
                    }
                ));

                ::near_sdk::env::storage_write(
                    self.pa_storage_key().as_ref(),
                    paused_keys
                        .try_to_vec()
                        .expect("Pausable: Unexpected error serializing keys")
                        .as_ref(),
                );
            }

            #[#cratename::only(owner)]
            fn pa_unpause_feature(&mut self, key: String) {
                let mut paused_keys = self.pa_all_paused().unwrap_or_default();
                paused_keys.remove(&key);

                ::near_sdk::log!(#cratename::events::AsEvent::event(
                    &#cratename::pausable::Unpause {
                        by: ::near_sdk::env::predecessor_account_id(),
                        key,
                    }
                ));

                if paused_keys.is_empty() {
                    ::near_sdk::env::storage_remove(self.pa_storage_key().as_ref());
                } else {
                    ::near_sdk::env::storage_write(
                        self.pa_storage_key().as_ref(),
                        paused_keys
                            .try_to_vec()
                            .expect("Pausable: Unexpected error serializing keys")
                            .as_ref(),
                    );
                }
            }
        }
    };

    output.into()
}

#[derive(Default, FromMeta, Debug)]
#[darling(default)]
pub struct ExceptSubArgs {
    #[darling(default)]
    owner: bool,
    #[darling(default)]
    #[darling(rename = "self")]
    _self: bool,
}

#[derive(Debug, FromMeta)]
pub struct PauseArgs {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    except: ExceptSubArgs,
}

pub fn pause(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();

    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }

    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let args = PauseArgs::from_list(&attr_args).expect("Invalid arguments");

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let stmts = &block.stmts;

    let fn_name = args.name.unwrap_or_else(|| sig.ident.to_string());

    let self_condition = if args.except._self {
        quote!(
            if ::near_sdk::env::predecessor_account_id() == ::near_sdk::env::current_account_id() {
                check_paused = false;
            }
        )
    } else {
        quote!()
    };

    let owner_condition = if args.except.owner {
        quote!(
            if Some(::near_sdk::env::predecessor_account_id()) == self.owner_get() {
                check_paused = false;
            }
        )
    } else {
        quote!()
    };

    let bypass_condition = quote!(
        #self_condition
        #owner_condition
    );

    let check_pause = quote!(
        let mut check_paused = true;
        #bypass_condition
        if check_paused {
            assert!(!self.pa_is_paused(#fn_name.to_string()), "Pausable: Method is paused");
        }
    );

    // https://stackoverflow.com/a/66851407
    let result = quote! {
        #(#attrs)* #vis #sig {
            #check_pause
            #(#stmts)*
        }
    };

    result.into()
}

#[derive(Debug, FromMeta)]
pub struct IfPausedArgs {
    name: String,
    #[darling(default)]
    except: ExceptSubArgs,
}

pub fn if_paused(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse::<ItemFn>(item.clone()).unwrap();

    if is_near_bindgen_wrapped_or_marshall(&input) {
        return item;
    }

    let attr_args = parse_macro_input!(attrs as AttributeArgs);
    let args = IfPausedArgs::from_list(&attr_args).expect("Invalid arguments");

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let stmts = &block.stmts;

    let fn_name = args.name;

    let self_condition = if args.except._self {
        quote!(
            if ::near_sdk::env::predecessor_account_id() == ::near_sdk::env::current_account_id() {
                check_paused = false;
            }
        )
    } else {
        quote!()
    };

    let owner_condition = if args.except.owner {
        quote!(
            if Some(::near_sdk::env::predecessor_account_id()) == self.owner_get() {
                check_paused = false;
            }
        )
    } else {
        quote!()
    };

    let bypass_condition = quote!(
        #self_condition
        #owner_condition
    );

    let check_pause = quote!(
        let mut check_paused = true;
        #bypass_condition
        if check_paused {
            assert!(self.pa_is_paused(#fn_name.to_string()), "Pausable: Method must be paused");
        }
    );

    // https://stackoverflow.com/a/66851407
    let result = quote! {
        #(#attrs)* #vis #sig {
            #check_pause
            #(#stmts)*
        }
    };

    result.into()
}
