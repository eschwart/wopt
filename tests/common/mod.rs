#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod params {
    pub const A: u8 = 69;
    pub const B: f32 = 420.0;
    pub const C: i32 = -2048;
}

mod util {
    pub const fn ser(data: &Vec<u8>) -> &[u8] {
        data.as_slice()
    }

    pub fn de(data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
}

pub mod named {
    pub use super::params::*;
    use wopt::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleNamed {
        pub a: u8,
        pub b: f32,
        pub c: i32,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleNamedReq {
        pub a: u8,
        #[wopt(required)]
        pub b: f32,
        pub c: i32,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleNamedWith {
        pub a: u8,
        pub b: f32,
        pub c: i32,
    }

    #[cfg(feature = "bytemuck")]
    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleNamedFlat {
        pub a: u8,
        #[wopt(optional, serde)]
        pub b: ExampleNamed,
        pub c: i32,
    }

    #[cfg(feature = "bytemuck")]
    #[derive(Clone, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleNamedVec {
        pub a: u8,
        #[wopt(ser = "super::util::ser", de = "super::util::de")]
        pub b: Vec<u8>,
        pub c: i32,
    }
}

pub mod unnamed {
    pub use super::params::*;
    use bytemuck::{Pod, Zeroable};
    use wopt::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnnamed(pub u8, pub f32, pub i32);

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnnamedReq(pub u8, #[wopt(required)] pub f32, pub i32);

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnnamedWith(pub u8, pub f32, pub i32);

    #[cfg(feature = "bytemuck")]
    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnnamedFlat(pub u8, #[wopt(optional, serde)] pub ExampleUnnamed, pub i32);

    #[cfg(feature = "bytemuck")]
    #[derive(Clone, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnnamedVec(
        pub u8,
        #[wopt(ser = "super::util::ser", de = "super::util::de")] pub Vec<u8>,
        pub i32,
    );
}

pub mod unit {
    use bytemuck::{Pod, Zeroable};
    use wopt::*;

    #[cfg(feature = "bytemuck")]
    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(derive(Debug, Default, PartialEq))]
    pub struct ExampleUnit;
}

// if all of this compiles, it works.
#[cfg(feature = "bf")]
mod bf {
    pub use super::params::*;
    use wopt::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(
        bf = "++++[++++>---<]>+.[-->+++<]>-.---.[--->+<]>-.++[->+++<]>++.-[--->+<]>--.++[->++<]>.[-->+++<]>-.+.-----.--[--->+<]>.---------.++++++++.[---->+<]>+++.[-->+++++<]>.[------->++<]>+.--[--->+<]>---.++.-----------.--------.+++++++++++.[->+++<]>+.-[--->+<]>+++++."
    )]
    pub struct ExampleNamed {
        pub a: u8,
        #[wopt(required)]
        pub b: f32,
        #[wopt(skip)]
        pub c: i32,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, WithOpt)]
    #[wopt(
        bf = "++++[++++>---<]>+.[-->+++<]>-.---.[--->+<]>-.++[->+++<]>++.-[--->+<]>--.++[->++<]>.[-->+++<]>-.+.-----.--[--->+<]>.---------.++++++++.[---->+<]>+++.[-->+++++<]>.[------->++<]>+.--[--->+<]>---.++.-----------.--------.+++++++++++.[->+++<]>+.-[--->+<]>+++++."
    )]
    pub struct ExampleUnnamed(pub u8, #[wopt(required)] pub f32, #[wopt(skip)] pub i32);
}
