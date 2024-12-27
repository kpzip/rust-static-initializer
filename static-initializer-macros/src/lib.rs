#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{Expr, ExprMacro, Ident, parse_macro_input, parse_quote, Token, Type, Visibility};
use syn::Expr::Macro;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

struct StaticWithInitializer {
    vis: Visibility,
    name: Ident,
    ty: Type,
    init: Expr,
}

impl Parse for StaticWithInitializer {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        input.parse::<Token![ref]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let init: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(StaticWithInitializer {
            vis,
            name,
            ty,
            init,
        })
    }
}

#[proc_macro_attribute]
pub fn static_init(attr: TokenStream, item: TokenStream) -> TokenStream {
    let StaticWithInitializer {
        vis,
        name,
        ty,
        init,
    } = parse_macro_input!(item as StaticWithInitializer);

    // usual assertions for static
    let assert_sync = quote_spanned! {ty.span()=>
        struct _AssertSync where #ty: std::marker::Sync;
    };
    let assert_sized = quote_spanned! {ty.span()=>
        struct _AssertSized where #ty: std::marker::Sized;
    };

    // User should set the static to equal unimplemented\
    // todo make this do path comparison?
    let mut warning = true;
    if let Macro(m) = init {
        if let Some(ident) = m.mac.path.get_ident() {
            if ident == "unimplemented" {
                // No Warning
                warning = false;
            }
        }
    }
    if warning {
        init.span().unwrap().warning("static with initializer should be set equal to `unimplemented!()`!").emit();
    }

    let expanded = quote! {
        #vis mod #name {

            #assert_sync
            #assert_sized

            static mut __STATIC_INIT_INTERNAL_STATIC_#name: core::mem::MaybeUninit<#ty> = core::mem::MaybeUninit::uninit();

            pub fn get_#name() -> &'static #ty {
                // SAFETY: initialized at the top of main
                unsafe {
                    (&*(&raw const __STATIC_INIT_INTERNAL_STATIC_#name)).assume_init_ref()
                }
            }
        }
    };

    return TokenStream::from(expanded);
}
