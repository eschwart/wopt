#![allow(dead_code)]

mod params {
    pub const A: u8 = 69;
    pub const B: f32 = 420.0;
    pub const C: i32 = -2048;
}

pub mod named {
    pub use super::params::*;
    use wopt::*;

    pub const EXAMPLE_NAMED: ExampleNamed = ExampleNamed { a: A, b: B, c: C };

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleNamed {
        pub a: u8,
        pub b: f32,
        pub c: i32,
    }
}

pub mod unnamed {
    pub use super::params::*;
    use wopt::*;

    pub const EXAMPLE_UNNAMED: ExampleUnnamed = ExampleUnnamed(A, B, C);

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnnamed(pub u8, pub f32, pub i32);
}
