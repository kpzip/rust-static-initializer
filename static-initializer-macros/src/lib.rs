#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{ExprBlock, Ident, parse_macro_input, Token, Type, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

struct StaticWithInitializer {
    vis: Visibility,
    name: Ident,
    ty: Type,
    init: ExprBlock,
}

impl Parse for StaticWithInitializer {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        input.parse::<Token![static]>()?;
        let init: ExprBlock = input.parse()?;
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

    let module_name = format_ident!("__static_init_module_n{}", name.to_string().to_lowercase());

    let expanded = quote! {
        #vis struct #name;

        pub mod #module_name {

            #assert_sync
            #assert_sized

            static mut INTERNAL: core::mem::MaybeUninit<#ty> = core::mem::MaybeUninit::uninit();

            #[allow(unused_braces)]
            pub unsafe fn init() {
                // SAFETY: this is the only place where it can be accessed mutably
                unsafe {
                    (&mut *(&raw mut INTERNAL)).write(#init);
                }
            }

            pub unsafe fn deinit() {
                // SAFETY: this is only called when the program exits
                unsafe {
                    (&mut *(&raw mut INTERNAL)).assume_init_drop();
                }
            }

            pub unsafe fn get_raw() -> *const core::mem::MaybeUninit<#ty> {
                &raw const INTERNAL
            }

        }

        impl std::ops::Deref for #name {
            type Target = #ty;

            fn deref(&self) -> &Self::Target {

                // SAFETY: initialized at the top of main
                unsafe {
                    (&*#module_name ::get_raw()).assume_init_ref()
                }
            }
        }
    };

    return TokenStream::from(expanded);
}
