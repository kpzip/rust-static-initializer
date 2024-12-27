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

#[proc_macro]
pub fn static_init(item: TokenStream) -> TokenStream {
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


    let expanded = quote! {
        #vis struct #name;

        impl std::ops::Deref for #name {
            type Target = #ty;

            fn deref(&self) -> &Self::Target {

                static mut INTERNAL: core::mem::MaybeUninit<#ty> = core::mem::MaybeUninit::uninit();

                // SAFETY: initialized at the top of main
                unsafe {
                    (&*(&raw const INTERNAL)).assume_init_ref()
                }
            }
        }
    };

    return TokenStream::from(expanded);
}
