mod common;
use common::unnamed::*;

#[test]
fn test_stable() {
    // base is_modified test
    let mut ex_opt = ExampleUnnamedOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (1, 2) of optional struct
    ex_opt.1 = Some(B);
    ex_opt.2 = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (0) of original struct
    let mut ex = ExampleUnnamed::default();
    ex.0 = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(
        ex,
        ExampleUnnamed {
            0: 69,
            1: 420.0,
            2: -2048
        }
    );

    // optional struct is now zeroed out
    assert_eq!(ex_opt, ExampleUnnamedOpt::default())
}

#[test]
fn test_stable_req() {
    // base is_modified test
    let mut ex_opt = ExampleUnnamedReqOpt::default();
    assert!(!ex_opt.is_modified());

    // modify (1, 2) of optional struct
    ex_opt.1 = B;
    ex_opt.2 = Some(C);
    assert!(ex_opt.is_modified());

    // instantiate, then modify (0) of original struct
    let mut ex = ExampleUnnamedReq::default();
    ex.0 = A;
    ex.patch(&mut ex_opt); // apply patch

    // optional struct should no longer be modified
    assert!(!ex_opt.is_modified());
    // original struct now contains all test parameters
    assert_eq!(
        ex,
        ExampleUnnamedReq {
            0: 69,
            1: 0.0,
            2: -2048
        }
    );

    // optional struct is now zeroed out
    assert_eq!(
        ex_opt,
        ExampleUnnamedReqOpt {
            0: None,
            1: 420.0,
            2: None
        }
    )
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize() {
    let mut ex_opt = ExampleUnnamedOpt::default();
    ex_opt.0 = Some(A);
    ex_opt.2 = Some(C);

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [0, 5, 69, 0, 248, 255, 255]);
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_serialize_req() {
    let mut ex_opt = ExampleUnnamedReqOpt::default();
    ex_opt.0 = Some(A);
    ex_opt.1 = B;

    let serialized = ex_opt.serialize();
    assert_eq!(serialized, [1, 1, 69, 0, 0, 210, 67]);
}

#[test]
#[cfg(feature = "rkyv")]
fn test_rkyv_deserialize() {
    let bytes = [0, 5, 69, 0, 248, 255, 255];

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
#[cfg(feature = "rkyv")]
fn test_rkyv_deserialize_req() {
    let bytes = [1, 1, 69, 0, 0, 210, 67];

    let deserialized = ExampleUnnamedReqOpt::deserialize(&bytes[1..]);
    assert_eq!(
        deserialized,
        ExampleUnnamedReqOpt {
            0: Some(A),
            1: B,
            2: None
        }
    );
}
