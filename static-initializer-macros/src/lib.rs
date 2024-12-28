use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{ExprBlock, Ident, LitStr, parse_macro_input, Token, Type, Visibility};
use syn::__private::{Span, TokenStream2};
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
        input.parse::<Token![unsafe]>()?;
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

fn cfg_windows() -> TokenStream2 {
    quote!(
        target_os = "windows"
    )
}

fn cfg_apple() -> TokenStream2 {
    quote!(
        any(target_os = "macos", target_os = "ios")
    )
}

fn cfg_unix() -> TokenStream2 {
    quote!(
        any(target_os = "linux", target_os = "android")
    )
}

fn cfg_unsupported() -> TokenStream2 {
    let win = cfg_windows();
    let apple = cfg_apple();
    let unix = cfg_unix();
    quote!(
        not(any(#win, #apple, #unix))
    )
}

fn get_initializer_attributes(priority: u16) -> TokenStream2 {
    let win = cfg_windows();
    let apple = cfg_apple();
    let unix = cfg_unix();
    let unsupported = cfg_unsupported();

    let win_sections = LitStr::new(format!(".CRT$XCU{:05}", priority).as_str(), Span::call_site());
    let apple_sections = LitStr::new("__DATA,__mod_init_func", Span::call_site());
    let unix_sections = LitStr::new(format!(".init_array.{:05}", priority).as_str(), Span::call_site());

    quote!(
        #[cfg(#unsupported)]
        compiler_error!("Unsupported Target OS!");
        // Linker Magic!
        #[cfg_attr(#win, unsafe(link_section = #win_sections))]
        #[cfg_attr(#apple, unsafe(link_section = #apple_sections))]
        #[cfg_attr(#unix, unsafe(link_section = #unix_sections))]
    )

}

fn get_deinitializer_attributes(priority: u16) -> TokenStream2 {
    let win = cfg_windows();
    let apple = cfg_apple();
    let unix = cfg_unix();
    let unsupported = cfg_unsupported();

    let win_sections = LitStr::new(format!(".CRT$XPTZ{:05}", priority).as_str(), Span::call_site());
    let apple_sections = LitStr::new("__DATA,__mod_term_func", Span::call_site());
    let unix_sections = LitStr::new(format!(".fini_array.{:05}", priority).as_str(), Span::call_site());

    quote!(
        #[cfg(#unsupported)]
        compiler_error!("Unsupported Target OS!");
        // Linker Magic!
        #[cfg_attr(#win, unsafe(link_section = #win_sections))]
        #[cfg_attr(#apple, unsafe(link_section = #apple_sections))]
        #[cfg_attr(#unix, unsafe(link_section = #unix_sections))]
    )

}

fn get_module_ident(var: &Ident) -> Ident {
    format_ident!("__static_init_module_n{}", var.to_string().to_lowercase())
}

/// # Global non-lazy zero-cost statics without `const fn`.
/// This macro is a safe* abstraction for initializing statics before `main()` is called. The initializer is called before `main()` and `Drop` is called after `main()` is finished.
/// # Syntax
/// > **<sup>Syntax</sup>**\
/// > _StaticItemWithInitializer_ :\
/// > &nbsp;&nbsp; `static_init!` { [Visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html)<sup>?</sup> `static` [Identifier](https://doc.rust-lang.org/reference/identifiers.html) `:` [Type](https://doc.rust-lang.org/reference/types.html#type-expressions)
/// >              ( `=` `unsafe` `static` [Block](https://doc.rust-lang.org/reference/expressions/block-expr.html) ) `;` }
/// >
///
/// # Undefined Behavior
/// *This macro may cause undefined behavior if:
/// - the initializer creates a new thread
/// - the initializer references other statics created with this macro
/// - the initializer references the static it is initializing (In violation of rust's aliasing rules)
/// - [`std::sync::mpmc`](https://doc.rust-lang.org/std/sync/mpmc/index.html) or [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html) is used
/// - See [Use before and after main](https://doc.rust-lang.org/std/#use-before-and-after-main)
///
/// For this reason, the unsafe keyword is required to declare initializers with this macro.
/// In the future these scenarios will hopefully become compile errors, and the unsafe keyword will no longer be required.
/// # Examples
/// ```rust
/// use static_initializer::static_init;
///
/// static_init! {
///     static TEST_STATIC: Vec<u8> = unsafe static { (0..15u8).collect() };
/// }
///
/// fn use_the_static() {
///     // Prints `Test vec: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]`
///     println!("Test vec: {:?}", TEST_STATIC.as_slice());
/// }
/// ```
/// # Compatibility
/// This macro only works on certain operating systems due to the fact that it uses link sections to run code before `main()`
/// All major operating systems are supported, and more may be supported in the future.
/// `wasm` is currently not supported.
/// # Under the hood
/// Internally this macro uses the `#[link_section]` attribute in order to have initializers and deinitializers run before and after `main()`
///
/// On windows the link section used is `.CRT$XCU<5 digit priority number>` for constructors and `.CRT$XPTZ<5 digit priority number>` for destructors.
///
/// On macOS and ios, `__DATA,__mod_init_func` and `__DATA,__mod_term_func` are used.
///
/// On linux and other Unix-based operating systems, `.init_array.<5 digit priority number>` and `.fini_array.<5 digit priority number>` are used.
///
/// Note `<5 digit priority number>` is replaced with a 5 digit base-10 formatted number ranging from `0` to [`u16::MAX`] which represents the order in which the initializers are run. Priority is not currently used and is not supported on some operating systems.
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

    let module_name = get_module_ident(&name);
    let priority: u16 = 65535;
    let init_attributes = get_initializer_attributes(priority);
    let deinit_attributes = get_deinitializer_attributes(priority);

    let expanded = quote! {
        #vis struct #name;

        #[doc(hidden)]
        mod #module_name {
            use super::*;

            #assert_sync
            #assert_sized

            static mut INTERNAL: core::mem::MaybeUninit<#ty> = core::mem::MaybeUninit::uninit();

            #[doc(hidden)]
            #[allow(unused_braces)]
            unsafe fn init() {
                // SAFETY: this is the only place where it can be accessed mutably
                unsafe {
                    (&mut *(&raw mut INTERNAL)).write(#init);
                }
            }

            #[doc(hidden)]
            unsafe fn deinit() {
                // SAFETY: this is only called when the program exits
                unsafe {
                    (&mut *(&raw mut INTERNAL)).assume_init_drop();
                }
            }

            #[doc(hidden)]
            pub unsafe fn get_raw() -> *const core::mem::MaybeUninit<#ty> {
                &raw const INTERNAL
            }

            // Add initializer fn pointers to the initializer array
            #init_attributes
            #[used]
            #[doc(hidden)]
            static _I: unsafe fn() -> () = init;

            #deinit_attributes
            #[used]
            #[doc(hidden)]
            static _D: unsafe fn() -> () = deinit;

        }

        #[doc(hidden)]
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
