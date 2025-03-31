use wopt::*;

#[allow(dead_code)]
#[derive(WithOpt)]
#[wopt(derive(Debug, Clone, PartialEq))]
struct Example {
    a: u8,
    b: f32,
    c: String,
}

#[test]
fn test_stable() {
    let b = 420.0;
    let mut ex_opt = ExampleOpt::default();
    ex_opt.b = Some(b);

    assert_eq!(
        ex_opt,
        ExampleOpt {
            a: None,
            b: Some(b),
            c: None
        },
    )
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv() {
    let a = 69;
    let mut ex_opt = ExampleOpt::default();
    ex_opt.a = Some(a);

    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&ex_opt).unwrap();
    let deserialized: ExampleOpt = rkyv::from_bytes::<_, rkyv::rancor::Error>(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized)
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let c = "very cool kanye".to_string();
    let mut ex_opt = ExampleOpt::default();
    ex_opt.c = Some(c);

    let serialized = serde_json::to_string(&ex_opt).unwrap();
    let deserialized: ExampleOpt = serde_json::from_str(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized)
}
