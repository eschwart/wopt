mod common;
use common::named::*;

#[test]
fn test_stable() {
    // base is_modified test
    let mut ex_opt = ExampleNamedOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (b, c) of optional struct
    ex_opt.b = Some(B);
    ex_opt.c = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (a) of original struct
    let mut ex = ExampleNamed::default();
    ex.a = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(ex, EXAMPLE_NAMED);

    // optional struct is now zeroed out
    assert_eq!(ex_opt, ExampleNamedOpt::default())
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize() {
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.a = Some(A);
    ex_opt.c = Some(C);

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [0, 5, 69, 0, 248, 255, 255]);
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_deserialize() {
    let bytes = [0, 5, 69, 0, 248, 255, 255];

    let deserialized = ExampleNamedOpt::deserialize(&bytes[1..]);
    assert_eq!(
        deserialized,
        ExampleNamedOpt {
            a: Some(A),
            b: None,
            c: Some(C)
        }
    );
}

#[test]
#[cfg(feature = "rkyv-full")]
fn test_rkyv_full() {
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.a = Some(A);

    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&ex_opt).unwrap();
    let deserialized: ExampleNamedOpt =
        rkyv::from_bytes::<_, rkyv::rancor::Error>(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized);
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let mut ex_opt = ExampleNamedOpt::default();
    ex_opt.a = Some(A);

    let serialized = serde_json::to_string(&ex_opt).unwrap();
    let deserialized: ExampleNamedOpt = serde_json::from_str(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized);
}
