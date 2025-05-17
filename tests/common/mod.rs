#![allow(dead_code)]
#![allow(unused_imports)]

mod params {
    pub const A: u8 = 69;
    pub const B: f32 = 420.0;
    pub const C: i32 = -2048;

    pub fn ser_f32(x: &f32) -> &[u8] {
        bytemuck::bytes_of(x)
    }

    pub fn de_f32(x: &[u8]) -> f32 {
        *bytemuck::from_bytes(x)
    }
}

pub mod named {
    pub use super::params::*;
    use bytemuck::{bytes_of, from_bytes};
    use wopt::*;

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 0))]
    pub struct ExampleNamed {
        pub a: u8,
        pub b: f32,
        pub c: i32,
    }

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 1))]
    pub struct ExampleNamedReq {
        pub a: u8,
        #[wopt(required)]
        pub b: f32,
        pub c: i32,
    }

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 2))]
    pub struct ExampleNamedWith {
        pub a: u8,
        #[wopt(ser = "super::params::ser_f32", de = "de_f32")]
        pub b: f32,
        pub c: i32,
    }
}

pub mod unnamed {
    pub use super::params::*;
    use wopt::*;

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 0))]
    pub struct ExampleUnnamed(pub u8, pub f32, pub i32);

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 1))]
    pub struct ExampleUnnamedReq(pub u8, #[wopt(required)] pub f32, pub i32);

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[cfg_attr(feature = "rkyv", wopt(id = 2))]
    pub struct ExampleUnnamedWith(
        pub u8,
        #[wopt(ser = "ser_f32", de = "de_f32")] pub f32,
        pub i32,
    );
}

pub mod unit {
    use wopt::*;

    #[cfg(feature = "rkyv")]
    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    #[wopt(id = 0)]
    pub struct ExampleUnit;
}

// if all of this compiles, it works.
#[cfg(feature = "bf")]
mod bf {
    pub use super::params::*;
    use wopt::*;

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(
        bf = "++++[++++>---<]>+.[-->+++<]>-.---.[--->+<]>-.++[->+++<]>++.-[--->+<]>--.++[->++<]>.[-->+++<]>-.+.-----.--[--->+<]>.---------.++++++++.[---->+<]>+++.[-->+++++<]>.[------->++<]>+.--[--->+<]>---.++.-----------.--------.+++++++++++.[->+++<]>+.-[--->+<]>+++++."
    )]
    #[cfg_attr(feature = "rkyv", wopt(id = 0))]
    pub struct ExampleNamed {
        pub a: u8,
        #[wopt(required)]
        pub b: f32,
        #[wopt(skip)]
        pub c: i32,
    }

    #[derive(Debug, Default, PartialEq, WithOpt)]
    #[wopt(
        bf = "++++[++++>---<]>+.[-->+++<]>-.---.[--->+<]>-.++[->+++<]>++.-[--->+<]>--.++[->++<]>.[-->+++<]>-.+.-----.--[--->+<]>.---------.++++++++.[---->+<]>+++.[-->+++++<]>.[------->++<]>+.--[--->+<]>---.++.-----------.--------.+++++++++++.[->+++<]>+.-[--->+<]>+++++."
    )]
    #[cfg_attr(feature = "rkyv", wopt(id = 1))]
    pub struct ExampleUnnamed(pub u8, #[wopt(required)] pub f32, #[wopt(skip)] pub i32);
}
