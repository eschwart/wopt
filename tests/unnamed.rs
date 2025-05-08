mod common;
use common::unnamed::*;

#[test]
fn test_stable() {
    // base is_modified test
    let mut ex_opt = ExampleUnnamedOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (b, c) of optional struct
    ex_opt.1 = Some(B);
    ex_opt.2 = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (a) of original struct
    let mut ex = ExampleUnnamed::default();
    ex.0 = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(ex, EXAMPLE_UNNAMED);

    // optional struct is now zeroed out
    assert_eq!(ex_opt, ExampleUnnamedOpt::default())
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize() {
    let mut ex_opt = ExampleUnnamedOpt::default();
    ex_opt.0 = Some(A);
    ex_opt.2 = Some(C);

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [1, 5, 69, 0, 248, 255, 255]);
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_deserialize() {
    let bytes = [1, 5, 69, 0, 248, 255, 255];

    let deserialized = ExampleUnnamedOpt::deserialize(&bytes[1..]);
    assert_eq!(
        deserialized,
        ExampleUnnamedOpt {
            0: Some(A),
            1: None,
            2: Some(C)
        }
    );
}

#[test]
#[cfg(feature = "rkyv-full")]
fn test_rkyv_full() {
    let mut ex_opt = ExampleUnnamedOpt::default();
    ex_opt.0 = Some(A);

    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&ex_opt).unwrap();
    let deserialized: ExampleUnnamedOpt =
        rkyv::from_bytes::<_, rkyv::rancor::Error>(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized);
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let mut ex_opt = ExampleUnnamedOpt::default();
    ex_opt.0 = Some(A);

    let serialized = serde_json::to_string(&ex_opt).unwrap();
    let deserialized: ExampleUnnamedOpt = serde_json::from_str(&serialized).unwrap();

    assert_eq!(ex_opt, deserialized);
}
