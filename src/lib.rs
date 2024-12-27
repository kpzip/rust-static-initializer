#![feature(raw_ref_op)]

pub use static_initializer_macros::static_init;

static_init! {
    static TEST: i32 = static { 1 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        todo!()
    }
}
