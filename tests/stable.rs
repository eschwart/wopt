#![allow(dead_code)]

use wopt::*;

#[derive(WithOpt)]
#[wopt(derive(Debug, Default, PartialEq))]
struct ExampleNamed {
    a: u8,
    #[wopt(required)]
    b: f32,
    c: String,
}

#[derive(WithOpt)]
#[wopt(derive(Debug, Default, PartialEq))]
struct ExampleUnnamed(f32);

#[test]
fn test_named() {
    let b = 420.0;
    let c = Some("very cool kanye".to_string());
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.b = b;
    ex_opt.c = c.clone();

    assert_eq!(ex_opt, ExampleNamedOpt { a: None, b, c },)
}

#[test]
fn test_unnamed() {
    let value = Some(420.0);
    let mut ex_opt = ExampleUnnamedOpt::default();
    ex_opt.0 = value;

    assert_eq!(ex_opt, ExampleUnnamedOpt(value),)
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv() {
    let a = 69;
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.a = Some(a);

    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&ex_opt).unwrap();
    let deserialized: ExampleNamedOpt =
        rkyv::from_bytes::<_, rkyv::rancor::Error>(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized)
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let c = "very cool kanye".to_string();
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.c = Some(c);

    let serialized = serde_json::to_string(&ex_opt).unwrap();
    let deserialized: ExampleNamedOpt = serde_json::from_str(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized)
}
