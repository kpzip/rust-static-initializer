#![cfg_attr(not(test), no_std)]
//! # Global non-lazy zero-cost statics without `const fn`.
//! Useful for static values that cannot be initialized through `const fn` but cannot incur the memory & performance cost of a [`std::sync::LazyLock`].
//!
//! - See [`static_init!`]
//!
//! # `no_std` support
//! this crate is `no_std`.

#[doc(inline)]
pub use static_initializer_macros::static_init;

#[cfg(test)]
mod tests {
    use super::*;

    static_init! {
        static TEST: Vec<u8> = unsafe static { (0..15u8).collect() };
    }

    #[test]
    fn it_works() {
        // Should cause UB if something is weird
        println!("Test vec: {:?}", TEST.as_slice());
    }
}
