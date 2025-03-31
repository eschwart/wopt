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
    let mut ex_opt = ExampleNamedOpt::default();
    assert!(!ex_opt.is_modified());

    let b = 420.0;
    let c = Some("very cool kanye".to_string());
    ex_opt.b = b;
    ex_opt.c = c.clone();
    assert!(ex_opt.is_modified());

    assert_eq!(ex_opt, ExampleNamedOpt { a: None, b, c });

    _ = ex_opt.take();
    assert!(!ex_opt.is_modified())
}

#[test]
fn test_unnamed() {
    let mut ex_opt = ExampleUnnamedOpt::default();
    assert!(!ex_opt.is_modified());

    let value = Some(420.0);
    ex_opt.0 = value;
    assert!(ex_opt.is_modified());

    assert_eq!(ex_opt, ExampleUnnamedOpt(value));

    _ = ex_opt.take();
    assert!(!ex_opt.is_modified())
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv() {
    let mut ex_opt = ExampleNamedOpt::default();
    assert!(!ex_opt.is_modified());

    let a = 69;
    ex_opt.a = Some(a);
    assert!(ex_opt.is_modified());

    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&ex_opt).unwrap();
    let deserialized: ExampleNamedOpt =
        rkyv::from_bytes::<_, rkyv::rancor::Error>(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized);

    _ = ex_opt.take();
    assert!(!ex_opt.is_modified())
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let mut ex_opt = ExampleNamedOpt::default();
    assert!(!ex_opt.is_modified());

    let c = "very cool kanye".to_string();
    ex_opt.c = Some(c);
    assert!(ex_opt.is_modified());

    let serialized = serde_json::to_string(&ex_opt).unwrap();
    let deserialized: ExampleNamedOpt = serde_json::from_str(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized);

    _ = ex_opt.take();
    assert!(!ex_opt.is_modified())
}
