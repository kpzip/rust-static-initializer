#![feature(raw_ref_op)]

pub use static_initializer_macros::static_init;

#[static_init]
static TEST: Vec<u8> = unimplemented!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        todo!()
    }
}
