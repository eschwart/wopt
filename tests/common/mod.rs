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
    #[cfg_attr(feature = "rkyv", wopt(id = 0))]
    pub struct ExampleNamed {
        pub a: u8,
        pub b: f32,
        pub c: i32,
    }

    pub const EXAMPLE_NAMED_REQ: ExampleNamedReq = ExampleNamedReq { a: A, b: B, c: C };

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 1))]
    pub struct ExampleNamedReq {
        pub a: u8,
        #[wopt(required)]
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
    #[cfg_attr(feature = "rkyv", wopt(id = 0))]
    pub struct ExampleUnnamed(pub u8, pub f32, pub i32);

    pub const EXAMPLE_UNNAMED_REA: ExampleUnnamedReq = ExampleUnnamedReq(A, B, C);

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 1))]
    pub struct ExampleUnnamedReq(pub u8, #[wopt(required)] pub f32, pub i32);
}
